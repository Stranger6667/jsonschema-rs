from typing import Any, TypeVar
from collections.abc import Iterator

_SchemaT = TypeVar('_SchemaT', bool, dict[str, Any])


def is_valid(
        schema: _SchemaT,
        instance: Any,
        draft: int | None = None,
        with_meta_schemas: bool | None = None
) -> bool:
    pass

def validate(
        schema: _SchemaT,
        instance: Any,
        draft: int | None = None,
        with_meta_schemas: bool | None = None
) -> None:
    pass

def iter_errors(
        schema: _SchemaT,
        instance: Any,
        draft: int | None = None,
        with_meta_schemas: bool | None = None
) -> Iterator[ValidationError]:
    pass


class JSONSchema:

    def __init__(
            self,
            schema: _SchemaT,
            draft: int | None = None,
            with_meta_schemas: bool | None = None
    ) -> None:
        pass

    @classmethod
    def from_str(
            cls,
            schema: str,
            draft: int | None = None,
            with_meta_schemas: bool | None = None
    ) -> 'JSONSchema':
        pass

    def is_valid(self, instance: Any) -> bool:
        pass

    def validate(self, instance: Any) -> None:
        pass

    def iter_errors(self, instance: Any) -> Iterator[ValidationError]:
        pass


class ValidationError(ValueError):
    message: str
    schema_path: list[str | int]
    instance_path: list[str | int]


Draft4: int
Draft6: int
Draft7: int
Draft201909: int
Draft202012: int
