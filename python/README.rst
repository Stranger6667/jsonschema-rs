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

**NOTE**. This library is in early development and not yet provide a way to show validation errors (even though it is implemented in the underlying Rust crate).

Performance
-----------

According to our benchmarks, ``jsonschema-rs`` is usually faster than existing alternatives in real-life scenarios.

However, for single-keyword or boolean schemas it might be slower than ``fastjsonschema``.

Compiled validators (when the input schema is compiled once and reused later)

+----------------+------------------------+----------------------+----------------------+------------------------+
| library        | ``false``              | ``{"minimum": 10}``  | small                | big                    |
+================+========================+======================+======================+========================+
| jsonschema-rs  |               320.3 ns |            329.32 ns |              1.15 us |                 5.8 ms |
+----------------+------------------------+----------------------+----------------------+------------------------+
| fastjsonschema |   52.29 ns (**x0.16**) | 134.43 ns (**x0.4**) |  6.01 us (**x5.22**) | 587.5 ms (**x101.29**) |
+----------------+------------------------+----------------------+----------------------+------------------------+
| jsonschema     |   289.97 ns (**x0.9**) |  2.52 us (**x7.65**) | 74.98 us (**x65.2**) |   2.02 s (**x348.27**) |
+----------------+------------------------+----------------------+----------------------+------------------------+

Validators are not compiled (``jsonschema``) or compiled on every validation:

+----------------+------------------------+-------------------------+-----------------------+-------------------------+
| library        | ``false``              | ``{"minimum": 10}``     | small                 | big                     |
+================+========================+=========================+=======================+=========================+
| jsonschema-rs  |              402.35 ns |               908.06 ns |               9.54 us |                  5.9 ms |
+----------------+------------------------+-------------------------+-----------------------+-------------------------+
| fastjsonschema | 64.08 us (**x159.26**) | 119.57 us (**x131.67**) | 1.43 ms (**x149.89**) | 599.84 ms (**x101.66**) |
+----------------+------------------------+-------------------------+-----------------------+-------------------------+
| jsonschema     | 67.74 us (**x168.36**) |   76.62 us (**x84.37**) | 1.02 ms (**x106.91**) |    2.11 s (**x357.62**) |
+----------------+------------------------+-------------------------+-----------------------+-------------------------+

The bigger the input is the bigger is performance win.

In the examples below, ``big`` and ``small`` schemas refer to more realistic schemas and input instances.
You can take a look at benchmarks in ``benches/bench.py``. Ratios are given against ``jsonschema-rs``.

Python support
--------------

``jsonschema-rs`` supports Python 3.5, 3.6, 3.7 and 3.8.

License
-------

The code in this project is licensed under `MIT license`_.
By contributing to ``jsonschema-rs``, you agree that your contributions
will be licensed under its MIT license.
 
.. |Build| image:: https://github.com/Stranger6667/jsonschema-rs/workflows/build/badge.svg
   :target: https://github.com/Stranger6667/jsonschema-rs/actions
.. |Version| image:: https://img.shields.io/pypi/v/jsonschema-rs.svg
   :target: https://pypi.org/project/jsonschema-rs/
.. |Python versions| image:: https://img.shields.io/pypi/pyversions/jsonschema-rs.svg
   :target: https://pypi.org/project/jsonschema-rs/
.. |License| image:: https://img.shields.io/pypi/l/jsonschema-rs.svg
   :target: https://opensource.org/licenses/MIT

.. _MIT license: https://opensource.org/licenses/MIT
