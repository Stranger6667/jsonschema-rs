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

However, for single-keyword or boolean schemas it might be slower than ``fastjsonschema`` or ``jsonschema`` on PyPy.

Schemas:

- ``kubernetes-openapi`` is an Open API schema for `Kubernetes <https://raw.githubusercontent.com/APIs-guru/openapi-directory/master/APIs/kubernetes.io/v1.10.0/swagger.yaml>`_ which is ~3.15 MB JSON file.
- ``small`` is taken from ``fastjsonschema`` benchmarks.

Compiled validators (when the input schema is compiled once and reused later). ``jsonschema-rs`` comes in two variants in the table below:

- ``validate``. This method raises ``ValidationError`` on errors or returns ``None`` on their absence.
- ```is_valid``. A faster method that returns a boolean result whether the instance is valid.

Ratios are given against the ``validate`` variant.

+-------------------------+------------------------+-----------------------+----------------------------+---------------------------+
| library                 | ``false``              |  ``{"minimum": 10}``  |  small                     |   kubernetes-openapi      |
+=========================+========================+=======================+============================+===========================+
| jsonschema-rs[validate] |              309.02 ns |             306.82 ns |                    1.36 us |                  21.59 ms |
+-------------------------+------------------------+-----------------------+----------------------------+---------------------------+
| jsonschema-rs[is_valid] |  180.98 ns (**x0.58**) | 190.82 ns (**x0.62**) |      947.06 ns (**x0.69**) |      18.26 ms (**x0.84**) |
+-------------------------+------------------------+-----------------------+----------------------------+---------------------------+
| fastjsonschema[CPython] |   59.89 ns (**x0.19**) | 107.55 ns (**x0.35**) |        4.31 us (**x3.16**) |      94.70 ms (**x4.38**) |
+-------------------------+------------------------+-----------------------+----------------------------+---------------------------+
| fastjsonschema[PyPy]    |   13.47 ns (**x0.04**) |  34.56 ns (**x0.11**) |        1.24 us (**x0.91**) |      49.48 ms (**x2.29**) |
+-------------------------+------------------------+-----------------------+----------------------------+---------------------------+
| jsonschema[CPython]     |  232.48 ns (**x0.75**) |   1.93 us (**x6.29**) |      59.16 us (**x43.50**) |       1.08 s (**x50.02**) |
+-------------------------+------------------------+-----------------------+----------------------------+---------------------------+
| jsonschema[PyPy]        |   42.44 ns (**x0.13**) | 257.79 ns (**x0.84**) |      26.51 us (**x19.49**) |    661.71 ms (**x30.64**) |
+-------------------------+------------------------+-----------------------+----------------------------+---------------------------+

The bigger the input is the bigger is performance win. You can take a look at benchmarks in ``benches/bench.py``.

Package versions:

- ``jsonschema-rs`` - latest version from the repository
- ``jsonschema`` - ``3.2.0``
- ``fastjsonschema`` - ``2.14.5``

Measured with stable Rust 1.49, CPython 3.9.1 / PyPy3 7.3.3 on i8700K (12 cores), 32GB RAM, Arch Linux.

Python support
--------------

``jsonschema-rs`` supports CPython 3.6, 3.7, 3.8 and 3.9.

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
