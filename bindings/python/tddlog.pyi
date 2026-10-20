__version__: str

class Error(Exception):
    """A program tddlog could not answer: the base of ParseError,
    GroundingError and CompilationError."""

class ParseError(Error):
    """The program is not in the language tddlog reads."""

class GroundingError(Error):
    """The program parses but cannot be ground: an unsupported construct or a
    rule that is not range restricted."""

class CompilationError(Error):
    """The ground program could not be compiled and counted under the vtree
    strategy asked for, or the well-founded model of some world leaves an
    atom undefined."""

class Solution:
    """The answers to a program's queries, and the probability of its
    evidence where ``solve`` was asked for it."""

    @property
    def answers(self) -> dict[str, float] | None:
        """Each query, as ProbLog prints it, to its probability, in the
        program's order; ``None`` where the evidence is inconsistent."""
    @property
    def evidence_probability(self) -> float | None:
        """P(e); ``None`` unless asked for."""
    @property
    def log10_evidence_probability(self) -> float | None:
        """The base-10 logarithm of P(e), exact beyond a float's range;
        ``None`` unless asked for."""

def solve(
    program: str,
    *,
    vtree: str = "auto",
    evidence_probability: bool = False,
    queries: bool = True,
) -> Solution: ...
