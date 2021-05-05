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
| jsonschema-rs[validate] |              200.82 ns |             203.10 ns |                    1.22 us |                    1.51 us |
+-------------------------+------------------------+-----------------------+----------------------------+----------------------------+
| jsonschema-rs[is_valid] |  187.60 ns (**x0.93**) | 185.24 ns (**x0.91**) |      850.25 ns (**x0.69**) |        1.18 us (**x0.78**) |
+-------------------------+------------------------+-----------------------+----------------------------+----------------------------+
| jsonschema-rs[overhead] |  180.83 ns (**x0.90**) | 181.68 ns (**x0.89**) |      638.40 ns (**x0.52**) |        1.06 us (**x0.70**) |
+-------------------------+------------------------+-----------------------+----------------------------+----------------------------+
| fastjsonschema[CPython] |   58.57 ns (**x0.29**) | 109.10 ns (**x0.53**) |        4.16 us (**x3.40**) |        4.75 us (**x3.14**) |
+-------------------------+------------------------+-----------------------+----------------------------+----------------------------+
| fastjsonschema[PyPy]    |   1.32 ns (**x0.006**) |  33.39 ns (**x0.16**) |        890 ns  (**x0.72**) |         875 ns (**x0.58**) |
+-------------------------+------------------------+-----------------------+----------------------------+----------------------------+
| jsonschema[CPython]     |  226.48 ns (**x1.12**) |   1.88 us (**x9.25**) |      56.58 us (**x46.37**) |      57.31 us (**x37.95**) |
+-------------------------+------------------------+-----------------------+----------------------------+----------------------------+
| jsonschema[PyPy]        |   41.18 ns (**x0.20**) | 224.94 ns (**x1.10**) |      23.40 us (**x19.18**) |      22.78 us (**x15.08**) |
+-------------------------+------------------------+-----------------------+----------------------------+----------------------------+

Large schemas:

+-------------------------+-------------------------+--------------------------+----------------------------+---------------------------+
| library                 | ``Zuora (OpenAPI)``     | ``Kubernetes (Swagger)`` | ``Canada (GeoJSON)``       | ``CITM catalog``          |
+=========================+=========================+==========================+============================+===========================+
| jsonschema-rs[validate] |               13.970 ms |                13.076 ms |                   4.428 ms |                  4.715 ms |
+-------------------------+-------------------------+--------------------------+----------------------------+---------------------------+
| jsonschema-rs[is_valid] |   13.664 ms (**x0.97**) |    11.506 ms (**x0.87**) |       4.422 ms (**x0.99**) |      3.134 ms (**x0.66**) |
+-------------------------+-------------------------+--------------------------+----------------------------+---------------------------+
| jsonschema-rs[overhead] |   12.206 ms (**x0.87**) |     8.116 ms (**x0.62**) |       3.666 ms (**x0.82**) |      2.648 ms (**x0.56**) |
+-------------------------+-------------------------+--------------------------+----------------------------+---------------------------+
| fastjsonschema[CPython] |                  -- (1) |    87.020 ms (**x6.65**) |      31.705 ms (**x7.16**) |     11.715 ms (**x2.48**) |
+-------------------------+-------------------------+--------------------------+----------------------------+---------------------------+
| fastjsonschema[PyPy]    |                  -- (1) |    38.586 ms (**x2.95**) |       8.417 ms (**x1.90**) |      4.789 ms (**x1.01**) |
+-------------------------+-------------------------+--------------------------+----------------------------+---------------------------+
| jsonschema[CPython]     | 749.615 ms (**x53.65**) |     1.032 s (**x78.92**) |      1.286 s (**x290.42**) |   112.510 ms (**x23.86**) |
+-------------------------+-------------------------+--------------------------+----------------------------+---------------------------+
| jsonschema[PyPy]        | 611.056 ms (**x43.74**) |  592.584 ms (**x45.31**) |   530.567 ms (**x119.82**) |     28.619 ms (**x6.06**) |
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
