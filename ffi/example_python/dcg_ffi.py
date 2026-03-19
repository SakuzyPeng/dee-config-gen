from __future__ import annotations

import ctypes as ct
import os
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Literal, Optional

DCG_STATUS_OK = 0
DCG_STATUS_INVALID_ARGUMENT = 1
DCG_STATUS_PARSE_ERROR = 2
DCG_STATUS_RESOLVE_ERROR = 3
DCG_STATUS_RENDER_ERROR = 4
DCG_STATUS_INTERNAL_ERROR = 5
DCG_STATUS_PANIC = 6

DCG_RENDER_FORMAT_XML = 0
DCG_RENDER_FORMAT_JSON = 1


class DcgStringView(ct.Structure):
    _fields_ = [("ptr", ct.POINTER(ct.c_uint8)), ("len", ct.c_size_t)]


class DcgOwnedString(ct.Structure):
    _fields_ = [("ptr", ct.POINTER(ct.c_uint8)), ("len", ct.c_size_t)]


class DcgResolveOptions(ct.Structure):
    _fields_ = [
        ("has_template_override", ct.c_bool),
        ("template_override", DcgStringView),
        ("allow_fixed_override", ct.c_bool),
        ("windows_drive", ct.c_uint8),
    ]


class DcgGenerateOptions(ct.Structure):
    _fields_ = [
        ("resolve", DcgResolveOptions),
        ("format", ct.c_uint32),
    ]


class DcgValidateOutput(ct.Structure):
    _fields_ = [
        ("template_id", DcgOwnedString),
        ("profile", DcgOwnedString),
        ("job_mode", DcgOwnedString),
        ("encode_mode", DcgOwnedString),
        ("output_container", DcgOwnedString),
    ]


class DcgGenerateOutput(ct.Structure):
    _fields_ = [
        ("rendered_config", DcgOwnedString),
        ("format", ct.c_uint32),
        ("template_id", DcgOwnedString),
        ("output_container", DcgOwnedString),
    ]


class DcgError(ct.Structure):
    _fields_ = [("code", ct.c_uint32), ("message", DcgOwnedString)]


@dataclass
class ValidateResult:
    template_id: str
    profile: str
    job_mode: str
    encode_mode: str
    output_container: str


@dataclass
class GenerateResult:
    rendered_config: str
    format: Literal["xml", "json"]
    template_id: str
    output_container: str


class DcgFfiError(RuntimeError):
    def __init__(self, code: int, message: str):
        self.code = code
        self.message = message
        super().__init__(f"dcg ffi failed: code={code}, message={message}")


def _detect_default_library() -> Path:
    repo_root = Path(__file__).resolve().parents[2]
    release_dir = repo_root / "target" / "release"
    candidates: list[Path]
    if sys.platform == "darwin":
        candidates = [release_dir / "libdee_config_gen.dylib"]
    elif os.name == "nt":
        candidates = [release_dir / "dee_config_gen.dll"]
    else:
        candidates = [release_dir / "libdee_config_gen.so"]

    for candidate in candidates:
        if candidate.exists():
            return candidate
    names = ", ".join(str(path) for path in candidates)
    raise FileNotFoundError(
        f"unable to locate FFI library. set DCG_FFI_LIB or build one of: {names}"
    )


def _view_from_utf8(text: str) -> tuple[DcgStringView, ct.Array[ct.c_char]]:
    encoded = text.encode("utf-8")
    buf = ct.create_string_buffer(encoded)
    view = DcgStringView(
        ptr=ct.cast(buf, ct.POINTER(ct.c_uint8)),
        len=len(encoded),
    )
    return view, buf


def _owned_string_to_text(value: DcgOwnedString) -> str:
    if not value.ptr or value.len == 0:
        return ""
    return ct.string_at(value.ptr, value.len).decode("utf-8", errors="replace")


class DcgFfiClient:
    def __init__(self, library_path: Optional[str] = None):
        if library_path is None:
            library_path = os.environ.get("DCG_FFI_LIB")
        if not library_path:
            library_path = str(_detect_default_library())
        self._lib = ct.CDLL(library_path)
        self._configure_signatures()

        abi_version = int(self._lib.dcg_abi_version())
        if abi_version != 1:
            raise RuntimeError(
                f"unsupported dcg abi version: {abi_version}, expected 1"
            )

    def _configure_signatures(self) -> None:
        self._lib.dcg_abi_version.argtypes = []
        self._lib.dcg_abi_version.restype = ct.c_uint32

        self._lib.dcg_validate_job.argtypes = [
            DcgStringView,
            ct.POINTER(DcgResolveOptions),
            ct.POINTER(DcgValidateOutput),
            ct.POINTER(DcgError),
        ]
        self._lib.dcg_validate_job.restype = ct.c_uint32

        self._lib.dcg_generate_config.argtypes = [
            DcgStringView,
            ct.POINTER(DcgGenerateOptions),
            ct.POINTER(DcgGenerateOutput),
            ct.POINTER(DcgError),
        ]
        self._lib.dcg_generate_config.restype = ct.c_uint32

        self._lib.dcg_free_validate_output.argtypes = [ct.POINTER(DcgValidateOutput)]
        self._lib.dcg_free_validate_output.restype = None
        self._lib.dcg_free_generate_output.argtypes = [ct.POINTER(DcgGenerateOutput)]
        self._lib.dcg_free_generate_output.restype = None
        self._lib.dcg_free_error.argtypes = [ct.POINTER(DcgError)]
        self._lib.dcg_free_error.restype = None

    def validate(self, job_text: str) -> ValidateResult:
        job_view, _job_buf = _view_from_utf8(job_text)
        out = DcgValidateOutput()
        err = DcgError()
        code = int(self._lib.dcg_validate_job(job_view, None, ct.byref(out), ct.byref(err)))

        try:
            if code != DCG_STATUS_OK:
                raise DcgFfiError(code, _owned_string_to_text(err.message))
            return ValidateResult(
                template_id=_owned_string_to_text(out.template_id),
                profile=_owned_string_to_text(out.profile),
                job_mode=_owned_string_to_text(out.job_mode),
                encode_mode=_owned_string_to_text(out.encode_mode),
                output_container=_owned_string_to_text(out.output_container),
            )
        finally:
            self._lib.dcg_free_validate_output(ct.byref(out))
            self._lib.dcg_free_error(ct.byref(err))

    def generate(
        self,
        job_text: str,
        format: Literal["xml", "json"] = "xml",
    ) -> GenerateResult:
        job_view, _job_buf = _view_from_utf8(job_text)
        out = DcgGenerateOutput()
        err = DcgError()
        options = DcgGenerateOptions()
        options.format = (
            DCG_RENDER_FORMAT_XML if format == "xml" else DCG_RENDER_FORMAT_JSON
        )

        code = int(
            self._lib.dcg_generate_config(
                job_view, ct.byref(options), ct.byref(out), ct.byref(err)
            )
        )

        try:
            if code != DCG_STATUS_OK:
                raise DcgFfiError(code, _owned_string_to_text(err.message))
            rendered_format = "xml" if out.format == DCG_RENDER_FORMAT_XML else "json"
            return GenerateResult(
                rendered_config=_owned_string_to_text(out.rendered_config),
                format=rendered_format,
                template_id=_owned_string_to_text(out.template_id),
                output_container=_owned_string_to_text(out.output_container),
            )
        finally:
            self._lib.dcg_free_generate_output(ct.byref(out))
            self._lib.dcg_free_error(ct.byref(err))
