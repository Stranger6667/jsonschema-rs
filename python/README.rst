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
| jsonschema-rs  |              152.02 ns |             151.25 ns |               711.97 ns |                 4.87 ms |
+----------------+------------------------+-----------------------+-------------------------+-------------------------+
| fastjsonschema |   49.76 ns (**x0.32**) | 128.64 ns (**x0.85**) |     5.51 us (**x7.74**) |  563.5 ms (**x115.57**) |
+----------------+------------------------+-----------------------+-------------------------+-------------------------+
| jsonschema     |  297.27 ns (**x1.95**) |  2.23 us (**x14.75**) |  73.95 us (**x103.87**) |    1.91 s (**x392.07**) |
+----------------+------------------------+-----------------------+-------------------------+-------------------------+

Validators are not compiled (``jsonschema``) or compiled on every validation:

+----------------+------------------------+-------------------------+-------------------------+-------------------------+
| library        | ``false``              | ``{"minimum": 10}``     |   small                 | big                     |
+================+========================+=========================+=========================+=========================+
| jsonschema-rs  |              344.44 ns |               470.42 ns |                  6.6 us |                 4.96 ms |
+----------------+------------------------+-------------------------+-------------------------+-------------------------+
| fastjsonschema | 62.24 us (**x180.71**) | 108.67 us (**x231.01**) |   1.31 ms (**x199.24**) |  561.6 ms (**x113.22**) |
+----------------+------------------------+-------------------------+-------------------------+-------------------------+
| jsonschema     | 62.93 us (**x182.72**) |  73.27 us (**x155.75**) | 931.65 us (**x141.16**) |    1.89 s (**x381.45**) |
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
