from typing import Any, Optional, TypeVar, Union
from collections.abc import Iterator

_SchemaT = TypeVar('_SchemaT', bool, dict[str, Any])


def is_valid(
        schema: _SchemaT,
        instance: Any,
        draft: Optional[int] = None,
        with_meta_schemas: Optional[bool] = None
) -> bool:
    pass

def validate(
        schema: _SchemaT,
        instance: Any,
        draft: Optional[int] = None,
        with_meta_schemas: Optional[bool] = None
) -> None:
    pass

def iter_errors(
        schema: _SchemaT,
        instance: Any,
        draft: Optional[int] = None,
        with_meta_schemas: Optional[bool] = None
) -> Iterator[ValidationError]:
    pass


class JSONSchema:

    def __init__(
            self,
            schema: _SchemaT,
            draft: Optional[int] = None,
            with_meta_schemas: Optional[bool] = None
    ) -> None:
        pass

    @classmethod
    def from_str(
            cls,
            schema: str,
            draft: Optional[int] = None,
            with_meta_schemas: Optional[bool] = None
    ) -> 'JSONSchema':
        pass

    def is_valid(self, instance: Any) -> bool:
        pass

    def validate(self, instance: Any) -> None:
        pass

    def iter_errors(self, instance: Any) -> Iterator[ValidationError]:
        pass


class ValidationError(Exception):
    message: str
    schema_path: list[Union[str, int]]
    instance_path: list[Union[str, int]]


Draft4: int
Draft6: int
Draft7: int
