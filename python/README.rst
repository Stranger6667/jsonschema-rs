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

+----------------+------------------------+-----------------------+-----------------------+-------------------------+
| library        | ``false``              |  ``{"minimum": 10}``  |  small                | big                     |
+================+========================+=======================+=======================+=========================+
| jsonschema-rs  |              273.01 ns |             263.29 ns |             904.38 ns |                 4.96 ms |
+----------------+------------------------+-----------------------+-----------------------+-------------------------+
| fastjsonschema |   49.77 ns (**x0.18**) | 130.21 ns (**x0.49**) |    5.7 us (**x6.31**) | 575.66 ms (**x115.84**) |
+----------------+------------------------+-----------------------+-----------------------+-------------------------+
| jsonschema     |  278.27 ns (**x1.01**) |   2.41 us (**x9.15**) | 75.27 us (**x83.23**) |    1.83 s (**x368.47**) |
+----------------+------------------------+-----------------------+-----------------------+-------------------------+

Validators are not compiled (``jsonschema``) or compiled on every validation:

+----------------+------------------------+-------------------------+-------------------------+-------------------------+
| library        | ``false``              | ``{"minimum": 10}``     |   small                 | big                     |
+================+========================+=========================+=========================+=========================+
| jsonschema-rs  |              349.74 ns |               465.67 ns |                 6.97 us |                 5.15 ms |
+----------------+------------------------+-------------------------+-------------------------+-------------------------+
| fastjsonschema | 62.46 us (**x178.59**) |  112.64 us (**x241.9**) |   1.33 ms (**x191.67**) |  574.5 ms (**x111.55**) |
+----------------+------------------------+-------------------------+-------------------------+-------------------------+
| jsonschema     | 64.57 us (**x184.64**) |   74.2 us (**x159.34**) | 951.38 us (**x136.37**) |    1.85 s (**x360.38**) |
+----------------+------------------------+-------------------------+-------------------------+-------------------------+

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
 
.. |Build| image:: https://github.com/Stranger6667/jsonschema-rs/workflows/ci/badge.svg
   :target: https://github.com/Stranger6667/jsonschema-rs/actions
.. |Version| image:: https://img.shields.io/pypi/v/jsonschema-rs.svg
   :target: https://pypi.org/project/jsonschema-rs/
.. |Python versions| image:: https://img.shields.io/pypi/pyversions/jsonschema-rs.svg
   :target: https://pypi.org/project/jsonschema-rs/
.. |License| image:: https://img.shields.io/pypi/l/jsonschema-rs.svg
   :target: https://opensource.org/licenses/MIT

.. _MIT license: https://opensource.org/licenses/MIT
