# tddlog for Python

```{include} ../README.md
:start-after: "</p>"
```

## Examples

`examples/alarm.py` is the program above; its docstring works the answers
out by hand:

```{literalinclude} ../examples/alarm.py
:language: python
```

`examples/graph.py` builds a program from Python data:

```{literalinclude} ../examples/graph.py
:language: python
```

The [guide](https://tractables.github.io/tddlog/guide/) describes the
language tddlog reads, with worked examples that open in the
[playground](https://tractables.github.io/tddlog/).

## Reference

```{eval-rst}
.. autofunction:: tddlog.solve

.. autoclass:: tddlog.Solution
   :members:

.. autoexception:: tddlog.Error

.. autoexception:: tddlog.ParseError

.. autoexception:: tddlog.GroundingError

.. autoexception:: tddlog.CompilationError
```
