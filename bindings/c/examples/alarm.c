/* The alarm example of Fierens et al. (2015): the probabilities of three
 * queries given that John calls, and the probability that he calls.
 *
 * P(alarm) = 1 - 0.9 * 0.8 = 0.28 and P(calls(john)) = 0.28 * 0.7 = 0.196,
 * so P(burglary | calls(john)) = 0.1 * 0.7 / 0.196. The program prints the
 * answers as the command line does, checks them against these numbers, and
 * prints
 *
 *   burglary:	0.357143
 *   earthquake:	0.714286
 *   calls(mary):	0.400000
 *   P(e):	0.196000	log10 -0.707744
 *
 * Building and linking it is covered in bindings/c/README.md. */

#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "tddlog.h"

static const char PROGRAM[] =
    "0.1::burglary.\n"
    "0.2::earthquake.\n"
    "0.7::hears_alarm(john).\n"
    "0.4::hears_alarm(mary).\n"
    "alarm :- burglary.\n"
    "alarm :- earthquake.\n"
    "calls(X) :- alarm, hears_alarm(X).\n"
    "evidence(calls(john), true).\n"
    "query(burglary).\n"
    "query(earthquake).\n"
    "query(calls(mary)).\n";

static void require(int holds, const char *what) {
  if (!holds) {
    fprintf(stderr, "alarm: %s\n", what);
    exit(1);
  }
}

int main(void) {
  TddlogOptions options = tddlog_options_default();
  options.evidence_probability = true;

  TddlogSolution *solution = NULL;
  if (tddlog_solve(PROGRAM, &options, &solution) != TDDLOG_OK) {
    fprintf(stderr, "alarm: %s\n", tddlog_last_error_message());
    return 1;
  }
  require(tddlog_solution_consistent(solution), "the evidence is consistent");

  for (size_t i = 0; i < tddlog_solution_num_answers(solution); i++) {
    printf("%s:\t%.6f\n", tddlog_solution_query(solution, i),
           tddlog_solution_probability(solution, i));
  }
  double evidence = 0.0, log10_evidence = 0.0;
  require(tddlog_solution_evidence_probability(solution, &evidence, &log10_evidence),
          "P(e) was asked for");
  printf("P(e):\t%.6f\tlog10 %.6f\n", evidence, log10_evidence);

  /* The answers against the numbers worked out above. */
  const double e = 0.28 * 0.7;
  const double expected[] = {0.1 * 0.7 / e, 0.2 * 0.7 / e, 0.4};
  const char *queries[] = {"burglary", "earthquake", "calls(mary)"};
  require(tddlog_solution_num_answers(solution) == 3, "three answers");
  for (size_t i = 0; i < 3; i++) {
    require(strcmp(tddlog_solution_query(solution, i), queries[i]) == 0, "the queries in order");
    require(fabs(tddlog_solution_probability(solution, i) - expected[i]) < 1e-12,
            "the answers worked out by hand");
  }
  require(fabs(evidence - e) < 1e-12, "P(e) = 0.196");
  require(fabs(log10_evidence - log10(e)) < 1e-12, "log10 P(e)");

  tddlog_solution_free(solution);
  return 0;
}
