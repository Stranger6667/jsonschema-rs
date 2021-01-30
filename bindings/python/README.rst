jsonschema-rs
=============

|Build| |Version| |Python versions| |License|

Fast JSON Schema validation for Python implemented in Rust.

Supported drafts:

- Draft 7
- Draft 6
- Draft 4

There are some notable restrictions at the moment:

- The underlying crate doesn't support arbitrary precision integers yet, which may lead to ``SystemError`` when such value is used;
- ``multipleOf`` keyword validation may produce false-negative results on some input. See `#84 <https://github.com/Stranger6667/jsonschema-rs/issues/84>`_ for more details

Installation
------------

To install ``jsonschema-rs`` via ``pip`` run the following command:

.. code:: bash

    pip install jsonschema-rs

Usage
-----

To check if the input document is valid:

.. code:: python

    import jsonschema_rs

    validator = jsonschema_rs.JSONSchema({"minimum": 42})
    validator.is_valid(45)  # True

or:

.. code:: python

    import jsonschema_rs

    validator = jsonschema_rs.JSONSchema({"minimum": 42})
    validator.validate(41)  # raises ValidationError

**NOTE**. This library is in early development.

Performance
-----------

According to our benchmarks, ``jsonschema-rs`` is usually faster than existing alternatives in real-life scenarios.

However, for single-keyword or boolean schemas it might be slower than ``fastjsonschema``.

Schemas:

- ``kubernetes-openapi`` is an Open API schema for `Kubernetes <https://raw.githubusercontent.com/APIs-guru/openapi-directory/master/APIs/kubernetes.io/v1.10.0/swagger.yaml>`_ which is ~3.15 MB JSON file.
- ``small`` is taken from ``fastjsonschema`` benchmarks.

Compiled validators (when the input schema is compiled once and reused later)

+-------------------------+------------------------+-----------------------+----------------------------+---------------------------+
| library                 | ``false``              |  ``{"minimum": 10}``  |  small                     |   kubernetes-openapi      |
+=========================+========================+=======================+============================+===========================+
| jsonschema-rs           |              192.53 ns |             268.73 ns |                  894.32 ns |                   27.2 ms |
+-------------------------+------------------------+-----------------------+----------------------------+---------------------------+
| fastjsonschema[CPython] |   56.75 ns (**x0.29**) |  108.13 ns (**x0.4**) |        4.24 us (**x4.74**) |                        \- |
+-------------------------+------------------------+-----------------------+----------------------------+---------------------------+
| fastjsonschema[PyPy]    |   12.65 ns (**x0.06**) |  29.93 ns (**x0.11**) |        1.24 us (**x1.38**) |                        \- |
+-------------------------+------------------------+-----------------------+----------------------------+---------------------------+
| jsonschema[CPython]     |  220.95 ns (**x1.14**) |   1.86 us (**x6.92**) |      58.83 us (**x65.78**) |      1.048 s (**x38.52**) |
+-------------------------+------------------------+-----------------------+----------------------------+---------------------------+
| jsonschema[PyPy]        |   41.73 ns (**x0.26**) |    293 ns (**x1.09**) |      25.71 us (**x28.74**) |    673.81 ms (**x24.77**) |
+-------------------------+------------------------+-----------------------+----------------------------+---------------------------+

The bigger the input is the bigger is performance win.
Unfortunately, ``fastjsonschema`` did not complete benchmarks for ``kubernetes-openapi`` due to an out-of-memory error.
However, on average the first run takes ~104ms on CPython and ~490ms on PyPy and later it increases exponentially.

You can take a look at benchmarks in ``benches/bench.py``. Ratios are given against ``jsonschema-rs``.
Measured with stable Rust 1.49, CPython 3.9.1 / PyPy3 7.3.3 on i8700K (12 cores), 32GB RAM, Arch Linux.

Python support
--------------

``jsonschema-rs`` supports Python 3.6, 3.7, 3.8 and 3.9.

License
-------

The code in this project is licensed under `MIT license`_.
By contributing to ``jsonschema-rs``, you agree that your contributions
will be licensed under its MIT license.
 
.. |Build| image:: https://github.com/Stranger6667/jsonschema-rs/workflows/ci/badge.svg
   :target: https://github.com/Stranger6667/jsonschema-rs/actions
.. |Version| image:: https://img.shields.io/pypi/v/jsonschema-rs.svg
   :target: https://pypi.org/project/jsonschema-rs/
.. |Python versions| image:: https://img.shields.io/pypi/pyversions/jsonschema-rs.svg
   :target: https://pypi.org/project/jsonschema-rs/
.. |License| image:: https://img.shields.io/pypi/l/jsonschema-rs.svg
   :target: https://opensource.org/licenses/MIT

.. _MIT license: https://opensource.org/licenses/MIT
