#ifndef DEE_CONFIG_GEN_FFI_H
#define DEE_CONFIG_GEN_FFI_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef uint32_t DcgStatusCode;
enum {
  DCG_STATUS_OK = 0,
  DCG_STATUS_INVALID_ARGUMENT = 1,
  DCG_STATUS_PARSE_ERROR = 2,
  DCG_STATUS_RESOLVE_ERROR = 3,
  DCG_STATUS_RENDER_ERROR = 4,
  DCG_STATUS_INTERNAL_ERROR = 5,
  DCG_STATUS_PANIC = 6
};

typedef uint32_t DcgRenderFormat;
enum {
  DCG_RENDER_FORMAT_XML = 0,
  DCG_RENDER_FORMAT_JSON = 1
};

typedef struct DcgStringView {
  const uint8_t *ptr;
  size_t len;
} DcgStringView;

typedef struct DcgOwnedString {
  uint8_t *ptr;
  size_t len;
} DcgOwnedString;

typedef struct DcgResolveOptions {
  bool has_template_override;
  DcgStringView template_override;
  bool allow_fixed_override;
  uint8_t windows_drive;
} DcgResolveOptions;

typedef struct DcgGenerateOptions {
  DcgResolveOptions resolve;
  DcgRenderFormat format;
} DcgGenerateOptions;

typedef struct DcgError {
  DcgStatusCode code;
  DcgOwnedString message;
} DcgError;

typedef struct DcgValidateOutput {
  DcgOwnedString template_id;
  DcgOwnedString profile;
  DcgOwnedString job_mode;
  DcgOwnedString encode_mode;
  DcgOwnedString output_container;
} DcgValidateOutput;

typedef struct DcgGenerateOutput {
  DcgOwnedString rendered_config;
  DcgRenderFormat format;
  DcgOwnedString template_id;
  DcgOwnedString output_container;
} DcgGenerateOutput;

uint32_t dcg_abi_version(void);

DcgStatusCode dcg_validate_job(
    DcgStringView job,
    const DcgResolveOptions *options,
    DcgValidateOutput *out,
    DcgError *err);

DcgStatusCode dcg_generate_config(
    DcgStringView job,
    const DcgGenerateOptions *options,
    DcgGenerateOutput *out,
    DcgError *err);

void dcg_free_string(DcgOwnedString *value);
void dcg_free_validate_output(DcgValidateOutput *out);
void dcg_free_generate_output(DcgGenerateOutput *out);
void dcg_free_error(DcgError *err);

#ifdef __cplusplus
}
#endif

#endif
