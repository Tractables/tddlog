/* tddlog: exact probabilities for ProbLog programs, by compilation into tree
 * decision diagrams.
 *
 * Copyright the tddlog authors. Licensed under the Apache License, Version 2.0.
 *
 * The ABI is unstable before 1.0: struct layouts, status codes and function
 * signatures can change in any release. Build against the tddlog version you
 * ship with, and check tddlog_version() at run time.
 */

#ifndef TDDLOG_H
#define TDDLOG_H

/* Generated from bindings/c/src/lib.rs by cbindgen. Do not edit by hand;
 * regenerate it with python bindings/c/check.py --write-header. */

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/**
 * The call succeeded.
 */
#define TDDLOG_OK 0

/**
 * The arguments broke a documented contract: a null pointer, text that is
 * not UTF-8, an unknown vtree strategy, or options that ask for nothing.
 */
#define TDDLOG_ERROR_INVALID_INPUT 1

/**
 * The program is not in the language tddlog reads.
 */
#define TDDLOG_ERROR_PARSE 2

/**
 * The program parses but cannot be ground: an unsupported construct or a
 * rule that is not range restricted.
 */
#define TDDLOG_ERROR_GROUND 3

/**
 * The ground program could not be compiled and counted under the vtree
 * strategy asked for, or the well-founded model of some world leaves an atom
 * undefined.
 */
#define TDDLOG_ERROR_COMPILE 4

/**
 * tddlog panicked. The panic did not cross into the caller; report it as a
 * bug.
 */
#define TDDLOG_ERROR_PANIC 5

/**
 * The answers to a program's queries, and `P(e)` where it was asked for.
 * `tddlog_solve` makes one and `tddlog_solution_free` releases it; the
 * accessors only read it, so threads may share one.
 */
typedef struct TddlogSolution TddlogSolution;

/**
 * What `tddlog_solve` is asked to do: the command line's options. Start from
 * `tddlog_options_default` and change what you need.
 */
typedef struct TddlogOptions {
  /**
   * The vtree strategy, the command line's `--vtree`: `"auto"`, or one of
   * the layouts `tddlog --help` lists. Null is `"auto"`.
   */
  const char *vtree;
  /**
   * Also compute `P(e)`, the probability of the evidence, which
   * `tddlog_solution_evidence_probability` then gives. The evidence no
   * query depends on is then compiled rather than dropped.
   */
  bool evidence_probability;
  /**
   * Answer the program's queries. `false` leaves them out, as the command
   * line's `--no-queries` does, and needs `evidence_probability`.
   */
  bool queries;
} TddlogOptions;

/**
 * What `tddlog_solve` returned: `TDDLOG_OK`, or one of the `TDDLOG_ERROR_`
 * values. `tddlog_last_error_message` describes the failure in words.
 */
typedef int32_t TddlogStatus;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * The version of these bindings, which is the version of the tddlog release
 * they wrap. The string is static and outlives every other call.
 */
const char *tddlog_version(void);

/**
 * Why the last `tddlog_solve` on this thread failed, as a NUL-terminated
 * string: the message the command line prints. The empty string where it
 * succeeded. Never null.
 *
 * The message belongs to tddlog and is replaced by the next `tddlog_solve`
 * on the same thread; copy it to keep it.
 */
const char *tddlog_last_error_message(void);

/**
 * The defaults: `"auto"`, the queries answered, no `P(e)`.
 */
struct TddlogOptions tddlog_options_default(void);

/**
 * Parse, ground, compile and count `program`, a ProbLog program as
 * NUL-terminated UTF-8 text, under `options` (null for the defaults).
 *
 * On `TDDLOG_OK`, `*out` is a solution the caller releases with
 * `tddlog_solution_free`; on any other status `*out` is null.
 *
 * # Safety
 *
 * `program` must point to a NUL-terminated string, `options` must be null or
 * point to one `TddlogOptions` whose `vtree` is null or a NUL-terminated
 * string, and `out` must point to storage for one pointer.
 */
TddlogStatus tddlog_solve(const char *program,
                          const struct TddlogOptions *options,
                          struct TddlogSolution **out);

/**
 * Release a solution `tddlog_solve` made. Null is accepted.
 *
 * # Safety
 *
 * `solution` must be null or a solution from `tddlog_solve` that nothing has
 * freed since.
 */
void tddlog_solution_free(struct TddlogSolution *solution);

/**
 * Whether the evidence is consistent. Where it is not, the solution has no
 * answers, and `P(e)`, where asked for, is zero.
 *
 * # Safety
 *
 * `solution` must be a live solution from `tddlog_solve`.
 */
bool tddlog_solution_consistent(const struct TddlogSolution *solution);

/**
 * The number of answers: one per query of the program, in its order; zero
 * where the evidence is inconsistent or the queries were left out.
 *
 * # Safety
 *
 * `solution` must be a live solution from `tddlog_solve`.
 */
size_t tddlog_solution_num_answers(const struct TddlogSolution *solution);

/**
 * The query of answer `index`, as ProbLog prints it (`\+p` for a
 * `query(\+p)`), or null where `index` is not below
 * `tddlog_solution_num_answers`. The string belongs to the solution and
 * lives until it is freed.
 *
 * # Safety
 *
 * `solution` must be a live solution from `tddlog_solve`.
 */
const char *tddlog_solution_query(const struct TddlogSolution *solution, size_t index);

/**
 * The probability of answer `index`, or NaN where `index` is not below
 * `tddlog_solution_num_answers`.
 *
 * # Safety
 *
 * `solution` must be a live solution from `tddlog_solve`.
 */
double tddlog_solution_probability(const struct TddlogSolution *solution, size_t index);

/**
 * `P(e)`, the probability of the evidence, where `options.evidence_probability`
 * asked for it: writes it to `*probability` and its base-10 logarithm, exact
 * beyond a double's range, to `*log10_probability`, and returns true. `P(e)`
 * is 1 with no evidence, 0 with inconsistent evidence (its logarithm
 * `-INFINITY`), and 0 too where it is under the least double. Returns false,
 * writing nothing, where it was not asked for. Either output may be null.
 *
 * # Safety
 *
 * `solution` must be a live solution from `tddlog_solve`, and each output
 * null or a pointer to storage for one double.
 */
bool tddlog_solution_evidence_probability(const struct TddlogSolution *solution,
                                          double *probability,
                                          double *log10_probability);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* TDDLOG_H */
