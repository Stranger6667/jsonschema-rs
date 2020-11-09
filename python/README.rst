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

Compiled validators (when the input schema is compiled once and reused later)

+----------------+------------------------+-----------------------+-------------------------+-------------------------+
| library        | ``false``              |  ``{"minimum": 10}``  |  small                  | big                     |
+================+========================+=======================+=========================+=========================+
| jsonschema-rs  |              141.45 ns |             144.66 ns |               652.84 ns |                 4.89 ms |
+----------------+------------------------+-----------------------+-------------------------+-------------------------+
| fastjsonschema |   48.92 ns (**x0.34**) |  95.22 ns (**x0.65**) |        3.91 us (**x6**) | 554.74 ms (**x113.44**) |
+----------------+------------------------+-----------------------+-------------------------+-------------------------+
| jsonschema     |  204.94 ns (**x1.44**) |   1.52 us (**10.54**) |      57.44 us (**x88**) |    1.38 s (**x282.41**) |
+----------------+------------------------+-----------------------+-------------------------+-------------------------+

Validators are not compiled (``jsonschema``) or compiled on every validation:

+----------------+------------------------+-------------------------+-------------------------+-------------------------+
| library        | ``false``              | ``{"minimum": 10}``     |   small                 | big                     |
+================+========================+=========================+=========================+=========================+
| jsonschema-rs  |              328.86 ns |               448.03 ns |                 6.39 us |                 4.89 ms |
+----------------+------------------------+-------------------------+-------------------------+-------------------------+
| fastjsonschema | 55.29 us (**x168.07**) |  106.01 us (**x236.6**) |    1.3 ms (**x204.53**) | 557.35 ms (**x113.97**) |
+----------------+------------------------+-------------------------+-------------------------+-------------------------+
| jsonschema     | 45.95 us (**x139.69**) |  54.68 us (**x122.06**) |  758.8 us (**x118.74**) |    1.43 s (**x292.43**) |
+----------------+------------------------+-------------------------+-------------------------+-------------------------+

The bigger the input is the bigger is performance win.

In the examples below, ``big`` and ``small`` schemas refer to more realistic schemas and input instances.
You can take a look at benchmarks in ``benches/bench.py``. Ratios are given against ``jsonschema-rs``.
Measured with stable Rust 1.44.1, Python 3.8.3 on i8700K (12 cores), 32GB RAM, Arch Linux.

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
