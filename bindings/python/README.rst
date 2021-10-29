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

If you have a schema as a JSON string, then you could use `jsonschema_rs.JSONSchema.from_str` to avoid parsing on the Python side:

.. code:: python

    import jsonschema_rs

    validator = jsonschema_rs.JSONSchema.from_str('{"minimum": 42}')
    ...

Performance
-----------

According to our benchmarks, ``jsonschema-rs`` is usually faster than existing alternatives in real-life scenarios.

However, for small schemas & inputs it might be slower than ``fastjsonschema`` or ``jsonschema`` on PyPy.

Input values and schemas
~~~~~~~~~~~~~~~~~~~~~~~~

- `Zuora <https://github.com/APIs-guru/openapi-directory/blob/master/APIs/zuora.com/2021-04-23/openapi.yaml>`_ OpenAPI schema (``zuora.json``). Validated against `OpenAPI 3.0 JSON Schema <https://github.com/OAI/OpenAPI-Specification/blob/main/schemas/v3.0/schema.json>`_ (``openapi.json``).
- `Kubernetes <https://raw.githubusercontent.com/APIs-guru/openapi-directory/master/APIs/kubernetes.io/v1.10.0/swagger.yaml>`_ Swagger schema (``kubernetes.json``). Validated against `Swagger JSON Schema <https://github.com/OAI/OpenAPI-Specification/blob/main/schemas/v2.0/schema.json>`_ (``swagger.json``).
- Canadian border in GeoJSON format (``canada.json``). Schema is taken from the `GeoJSON website <https://geojson.org/schema/FeatureCollection.json>`_ (``geojson.json``).
- Concert data catalog (``citm_catalog.json``). Schema is inferred via `infers-jsonschema <https://github.com/Stranger6667/infers-jsonschema>`_ & manually adjusted (``citm_catalog_schema.json``).
- ``Fast`` is taken from `fastjsonschema benchmarks <https://github.com/horejsek/python-fastjsonschema/blob/master/performance.py#L15>`_ (``fast_schema.json``, `f`ast_valid.json`` and ``fast_invalid.json``).

+----------------+-------------+---------------+
| Case           | Schema size | Instance size |
+================+=============+===============+
| OpenAPI        | 18 KB       | 4.5 MB        |
+----------------+-------------+---------------+
| Swagger        | 25 KB       | 3.0 MB        |
+----------------+-------------+---------------+
| Canada         | 4.8 KB      | 2.1 MB        |
+----------------+-------------+---------------+
| CITM catalog   | 2.3 KB      | 501 KB        |
+----------------+-------------+---------------+
| Fast (valid)   | 595 B       | 55 B          |
+----------------+-------------+---------------+
| Fast (invalid) | 595 B       | 60 B          |
+----------------+-------------+---------------+

Compiled validators (when the input schema is compiled once and reused later). ``jsonschema-rs`` comes in three variants in the tables below:

- ``validate``. This method raises ``ValidationError`` on errors or returns ``None`` on their absence.
- ``is_valid``. A faster method that returns a boolean result whether the instance is valid.
- ``overhead``. Only transforms data to underlying Rust types and do not perform any validation. Shows the Python -> Rust data conversion cost.

Ratios are given against the ``validate`` variant.

Small schemas:

+-------------------------+------------------------+-----------------------+----------------------------+----------------------------+
| library                 | ``true``               | ``{"minimum": 10}``   | ``Fast (valid)``           | ``Fast (invalid)``         |
+=========================+========================+=======================+============================+============================+
| jsonschema-rs[validate] |               80.83 ns |              86.23 ns |                  982.01 ns |                    1.54 us |
+-------------------------+------------------------+-----------------------+----------------------------+----------------------------+
| jsonschema-rs[is_valid] |   68.29 ns (**x0.84**) |  71.66 ns (**x0.83**) |      650.68 ns (**x0.66**) |        1.25 us (**x0.81**) |
+-------------------------+------------------------+-----------------------+----------------------------+----------------------------+
| jsonschema-rs[overhead] |   65.27 ns (**x0.81**) |  66.90 ns (**x0.78**) |      461.53 ns (**x0.47**) |      925.16 ns (**x0.60**) |
+-------------------------+------------------------+-----------------------+----------------------------+----------------------------+
| fastjsonschema[CPython] |   58.57 ns (**x0.72**) | 109.10 ns (**x1.27**) |        4.16 us (**x4.24**) |        4.75 us (**x3.08**) |
+-------------------------+------------------------+-----------------------+----------------------------+----------------------------+
| fastjsonschema[PyPy]    |    1.32 ns (**x0.02**) |  33.39 ns (**x0.39**) |        890 ns  (**x0.91**) |         875 ns (**x0.57**) |
+-------------------------+------------------------+-----------------------+----------------------------+----------------------------+
| jsonschema[CPython]     |  226.48 ns (**x2.80**) |   1.88 us (**x21.8**) |      56.58 us (**x57.62**) |      57.31 us (**x37.21**) |
+-------------------------+------------------------+-----------------------+----------------------------+----------------------------+
| jsonschema[PyPy]        |   41.18 ns (**x0.51**) | 224.94 ns (**x2.61**) |      23.40 us (**x23.83**) |      22.78 us (**x14.79**) |
+-------------------------+------------------------+-----------------------+----------------------------+----------------------------+

Large schemas:

+-------------------------+-------------------------+--------------------------+----------------------------+---------------------------+
| library                 | ``Zuora (OpenAPI)``     | ``Kubernetes (Swagger)`` | ``Canada (GeoJSON)``       | ``CITM catalog``          |
+=========================+=========================+==========================+============================+===========================+
| jsonschema-rs[validate] |               17.431 ms |                13.861 ms |                   4.782 ms |                  4.551 ms |
+-------------------------+-------------------------+--------------------------+----------------------------+---------------------------+
| jsonschema-rs[is_valid] |   16.732 ms (**x0.96**) |    12.174 ms (**x0.88**) |       4.591 ms (**x0.96**) |      2.935 ms (**x0.64**) |
+-------------------------+-------------------------+--------------------------+----------------------------+---------------------------+
| jsonschema-rs[overhead] |   12.017 ms (**x0.69**) |     8.005 ms (**x0.58**) |       3.702 ms (**x0.77**) |      2.303 ms (**x0.51**) |
+-------------------------+-------------------------+--------------------------+----------------------------+---------------------------+
| fastjsonschema[CPython] |                  -- (1) |    87.020 ms (**x6.28**) |       31.705 ms (**6.63**) |     11.715 ms (**x2.57**) |
+-------------------------+-------------------------+--------------------------+----------------------------+---------------------------+
| fastjsonschema[PyPy]    |                  -- (1) |    38.586 ms (**x2.78**) |       8.417 ms (**x1.76**) |      4.789 ms (**x1.05**) |
+-------------------------+-------------------------+--------------------------+----------------------------+---------------------------+
| jsonschema[CPython]     | 749.615 ms (**x43.00**) |     1.032 s (**x74.45**) |      1.286 s (**x268.93**) |   112.510 ms (**x24.72**) |
+-------------------------+-------------------------+--------------------------+----------------------------+---------------------------+
| jsonschema[PyPy]        | 611.056 ms (**x35.06**) |  592.584 ms (**x42.75**) |   530.567 ms (**x110.95**) |     28.619 ms (**x6.07**) |
+-------------------------+-------------------------+--------------------------+----------------------------+---------------------------+

Notes:

1. ``fastjsonschema`` fails to compile the Open API spec due to the presence of the ``uri-reference`` format (that is not defined in Draft 4). However, unknown formats are `explicitly supported <https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-7.1>`_ by the spec.

The bigger the input is the bigger is performance win. You can take a look at benchmarks in ``benches/bench.py``.

Package versions:

- ``jsonschema-rs`` - latest version from the repository
- ``jsonschema`` - ``3.2.0``
- ``fastjsonschema`` - ``2.15.0``

Measured with stable Rust 1.51, CPython 3.9.4 / PyPy3 7.3.4 on i8700K (12 cores), 32GB RAM, Arch Linux.

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
