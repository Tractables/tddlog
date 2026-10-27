/* The C API's answers against ProbLog's and against numbers worked out by
 * hand, and its errors.
 *
 * Each argument is one of ProbLog's system tests (tests/problog in the
 * repository), whose `%Expected outcome:` comment block lists ProbLog's
 * probability for each query. Every program is solved under several vtree
 * strategies, which must all agree with ProbLog. */

#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "tddlog.h"

static int failures = 0;

#define CHECK(cond, ...)                                                       \
  do {                                                                         \
    if (!(cond)) {                                                             \
      fprintf(stderr, "%s:%d: ", __FILE__, __LINE__);                          \
      fprintf(stderr, __VA_ARGS__);                                            \
      fprintf(stderr, "\n");                                                   \
      failures++;                                                              \
    }                                                                          \
  } while (0)

static const char *STRATEGIES[] = {"auto", "td", "closed", "dynamic", "program"};

static char *read_file(const char *path) {
  FILE *file = fopen(path, "rb");
  if (!file) return NULL;
  fseek(file, 0, SEEK_END);
  long size = ftell(file);
  fseek(file, 0, SEEK_SET);
  char *text = malloc((size_t)size + 1);
  if (text && fread(text, 1, (size_t)size, file) != (size_t)size) {
    free(text);
    text = NULL;
  }
  if (text) text[size] = '\0';
  fclose(file);
  return text;
}

/* `text` without its whitespace, in place. */
static void squeeze(char *text) {
  char *to = text;
  for (const char *from = text; *from; from++)
    if (*from != ' ' && *from != '\t' && *from != '\r') *to++ = *from;
  *to = '\0';
}

/* The probability `solution` gives `query`, or NaN where it has none. */
static double answer(const TddlogSolution *solution, const char *query) {
  for (size_t i = 0; i < tddlog_solution_num_answers(solution); i++) {
    char text[256];
    snprintf(text, sizeof text, "%s", tddlog_solution_query(solution, i));
    squeeze(text);
    if (strcmp(text, query) == 0) return tddlog_solution_probability(solution, i);
  }
  return NAN;
}

/* Solve the program at `path` under `vtree` and compare each answer with
 * the `%Expected outcome:` block: lines `% <query> <probability>` up to the
 * first line that is not a comment or does not end in a number. */
static void check_program(const char *path, const char *vtree) {
  char *source = read_file(path);
  CHECK(source != NULL, "cannot read %s", path);
  if (!source) return;
  TddlogOptions options = tddlog_options_default();
  options.vtree = vtree;
  TddlogSolution *solution = NULL;
  TddlogStatus status = tddlog_solve(source, &options, &solution);
  CHECK(status == TDDLOG_OK, "%s under %s: status %d, %s", path, vtree, status,
        tddlog_last_error_message());
  if (status != TDDLOG_OK) {
    free(source);
    return;
  }
  CHECK(tddlog_solution_consistent(solution), "%s: inconsistent evidence", path);

  size_t expected = 0;
  char *block = strstr(source, "Expected outcome:");
  CHECK(block != NULL, "%s states no expected outcome", path);
  char *line = block ? strchr(block, '\n') : NULL;
  while (line && *++line == '%') {
    char *end = strchr(line, '\n');
    if (end) *end = '\0';
    char *body = line + strspn(line, "% \t");
    size_t length = strlen(body);
    while (length > 0 && strchr(" \t\r", body[length - 1])) body[--length] = '\0';
    if (length == 0) {
      line = end;
      continue;
    }
    char *last = NULL;
    for (char *c = body; *c; c++)
      if (*c == ' ' || *c == '\t') last = c;
    char *number_end = NULL;
    double probability = last ? strtod(last + 1, &number_end) : 0.0;
    if (!last || *number_end != '\0') break;
    *last = '\0';
    squeeze(body);
    double got = answer(solution, body);
    CHECK(fabs(got - probability) < 1e-6, "%s under %s: %s is %.17g, ProbLog says %.17g",
          path, vtree, body, got, probability);
    expected++;
    line = end;
  }
  CHECK(expected > 0, "%s: no expected answers read", path);
  tddlog_solution_free(solution);
  free(source);
}

static void check_evidence(void) {
  static const char alarm[] =
      "0.1::burglary. 0.2::earthquake. 0.7::hears_alarm(john).\n"
      "alarm :- burglary. alarm :- earthquake.\n"
      "calls(X) :- alarm, hears_alarm(X).\n"
      "evidence(calls(john), true). query(burglary).\n";
  TddlogOptions options = tddlog_options_default();
  TddlogSolution *solution = NULL;
  double p = -1.0, log10_p = -1.0;

  /* Not asked for: nothing is written. */
  CHECK(tddlog_solve(alarm, NULL, &solution) == TDDLOG_OK, "%s", tddlog_last_error_message());
  CHECK(!tddlog_solution_evidence_probability(solution, &p, &log10_p), "P(e) not asked for");
  CHECK(p == -1.0 && log10_p == -1.0, "nothing written where P(e) was not asked for");
  CHECK(strcmp(tddlog_last_error_message(), "") == 0, "no message after a success");
  tddlog_solution_free(solution);

  /* P(e) alone, the queries left out: P(alarm) * 0.7 = 0.28 * 0.7. */
  options.evidence_probability = true;
  options.queries = false;
  CHECK(tddlog_solve(alarm, &options, &solution) == TDDLOG_OK, "%s", tddlog_last_error_message());
  CHECK(tddlog_solution_num_answers(solution) == 0, "no answers without the queries");
  CHECK(tddlog_solution_evidence_probability(solution, &p, &log10_p), "P(e) asked for");
  CHECK(fabs(p - 0.196) < 1e-12, "P(e) is %.17g, not 0.196", p);
  CHECK(fabs(log10_p - log10(0.196)) < 1e-12, "log10 P(e) is %.17g", log10_p);
  tddlog_solution_free(solution);

  /* Inconsistent evidence: no answers, P(e) zero. */
  options.queries = true;
  CHECK(tddlog_solve("0.3::a. 0.4::b. c :- a, b. evidence(a, false). evidence(c, true). query(b).",
                     &options, &solution) == TDDLOG_OK,
        "%s", tddlog_last_error_message());
  CHECK(!tddlog_solution_consistent(solution), "inconsistent evidence");
  CHECK(tddlog_solution_num_answers(solution) == 0, "no answers with inconsistent evidence");
  CHECK(tddlog_solution_evidence_probability(solution, &p, &log10_p), "P(e) asked for");
  CHECK(p == 0.0 && isinf(log10_p) && log10_p < 0, "P(e) zero and its logarithm -inf");
  tddlog_solution_free(solution);

  /* 400 observations of probability 1/10: P(e) = 1e-400, under the least
     double, whose logarithm is still exact. */
  static char many[400 * 40 + 64];
  size_t used = 0;
  for (int i = 0; i < 400; i++)
    used += (size_t)snprintf(many + used, sizeof many - used, "0.1::f(%d). evidence(f(%d), true).\n", i, i);
  snprintf(many + used, sizeof many - used, "0.5::q. query(q).\n");
  CHECK(tddlog_solve(many, &options, &solution) == TDDLOG_OK, "%s", tddlog_last_error_message());
  CHECK(fabs(tddlog_solution_probability(solution, 0) - 0.5) < 1e-12, "P(q | e) = 0.5");
  CHECK(tddlog_solution_evidence_probability(solution, &p, &log10_p), "P(e) asked for");
  CHECK(p == 0.0 && fabs(log10_p + 400.0) < 1e-9, "log10 P(e) is %.17g, not -400", log10_p);
  tddlog_solution_free(solution);
}

static void check_accessors(void) {
  TddlogSolution *solution = NULL;
  CHECK(tddlog_solve("0.3::p. query(\\+p). query(p).", NULL, &solution) == TDDLOG_OK, "%s",
        tddlog_last_error_message());
  CHECK(tddlog_solution_num_answers(solution) == 2, "two answers");
  CHECK(strcmp(tddlog_solution_query(solution, 0), "\\+p") == 0, "a negative query is \\+p");
  CHECK(fabs(tddlog_solution_probability(solution, 0) - 0.7) < 1e-12, "P(\\+p) = 0.7");
  CHECK(tddlog_solution_query(solution, 2) == NULL, "no query past the last");
  CHECK(isnan(tddlog_solution_probability(solution, 2)), "no probability past the last");
  tddlog_solution_free(solution);
  tddlog_solution_free(NULL);
}

static void check_errors(void) {
  TddlogOptions options = tddlog_options_default();
  /* Not a solution: a failing call overwrites it with NULL. */
  static char sentinel;
  TddlogSolution *solution = NULL;
  struct {
    const char *program;
    const char *vtree;
    bool queries;
    TddlogStatus status;
    const char *message;
  } cases[] = {
      {"0.3::a query(a).", NULL, true, TDDLOG_ERROR_PARSE, "line 1"},
      {"p :- X > 1. query(p).", NULL, true, TDDLOG_ERROR_GROUND, "grounding error"},
      {"0.5::e. a :- \\+b. b :- \\+a. query(a).", NULL, true, TDDLOG_ERROR_COMPILE, "well-founded"},
      {"0.3::a. query(a).", "balanced", true, TDDLOG_ERROR_INVALID_INPUT, "unknown vtree strategy"},
      {"0.3::a. query(a).", NULL, false, TDDLOG_ERROR_INVALID_INPUT, "nothing to answer"},
      {NULL, NULL, true, TDDLOG_ERROR_INVALID_INPUT, "program must not be null"},
  };
  for (size_t i = 0; i < sizeof cases / sizeof cases[0]; i++) {
    options.vtree = cases[i].vtree;
    options.queries = cases[i].queries;
    solution = (TddlogSolution *)(void *)&sentinel;
    TddlogStatus status = tddlog_solve(cases[i].program, &options, &solution);
    CHECK(status == cases[i].status, "case %zu: status %d, not %d (%s)", i, status, cases[i].status,
          tddlog_last_error_message());
    CHECK(solution == NULL, "case %zu: no solution after an error", i);
    CHECK(strstr(tddlog_last_error_message(), cases[i].message) != NULL,
          "case %zu: the message `%s` does not say `%s`", i, tddlog_last_error_message(),
          cases[i].message);
  }
  CHECK(tddlog_solve("query(a).", NULL, NULL) == TDDLOG_ERROR_INVALID_INPUT, "out is null");
}

int main(int argc, char **argv) {
  printf("tddlog %s\n", tddlog_version());
  CHECK(argc > 1, "usage: answers PROGRAM.pl...");
  for (int i = 1; i < argc; i++)
    for (size_t s = 0; s < sizeof STRATEGIES / sizeof STRATEGIES[0]; s++)
      check_program(argv[i], STRATEGIES[s]);
  check_evidence();
  check_accessors();
  check_errors();
  if (failures) {
    fprintf(stderr, "%d checks failed\n", failures);
    return 1;
  }
  printf("%d programs under %zu strategies agree with ProbLog; all checks passed\n", argc - 1,
         sizeof STRATEGIES / sizeof STRATEGIES[0]);
  return 0;
}
