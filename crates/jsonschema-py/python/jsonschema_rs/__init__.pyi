from collections.abc import Iterator
from typing import Any, Callable, TypeVar

_SchemaT = TypeVar("_SchemaT", bool, dict[str, Any])
_FormatFunc = TypeVar("_FormatFunc", bound=Callable[[str], bool])

def is_valid(
    schema: _SchemaT,
    instance: Any,
    draft: int | None = None,
    with_meta_schemas: bool | None = None,
    formats: dict[str, _FormatFunc] | None = None,
) -> bool:
    pass

def validate(
    schema: _SchemaT,
    instance: Any,
    draft: int | None = None,
    with_meta_schemas: bool | None = None,
    formats: dict[str, _FormatFunc] | None = None,
) -> None:
    pass

def iter_errors(
    schema: _SchemaT,
    instance: Any,
    draft: int | None = None,
    with_meta_schemas: bool | None = None,
    formats: dict[str, _FormatFunc] | None = None,
) -> Iterator[ValidationError]:
    pass

class JSONSchema:
    def __init__(
        self,
        schema: _SchemaT,
        draft: int | None = None,
        with_meta_schemas: bool | None = None,
        formats: dict[str, _FormatFunc] | None = None,
    ) -> None:
        pass

    @classmethod
    def from_str(
        cls,
        schema: str,
        draft: int | None = None,
        with_meta_schemas: bool | None = None,
        formats: dict[str, _FormatFunc] | None = None,
    ) -> "JSONSchema":
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

class Draft4Validator:
    def __init__(
        self,
        schema: _SchemaT | str,
        formats: dict[str, _FormatFunc] | None = None,
    ) -> None:
        pass

    def is_valid(self, instance: Any) -> bool:
        pass

    def validate(self, instance: Any) -> None:
        pass

    def iter_errors(self, instance: Any) -> Iterator[ValidationError]:
        pass

class Draft6Validator:
    def __init__(
        self,
        schema: _SchemaT | str,
        formats: dict[str, _FormatFunc] | None = None,
    ) -> None:
        pass

    def is_valid(self, instance: Any) -> bool:
        pass

    def validate(self, instance: Any) -> None:
        pass

    def iter_errors(self, instance: Any) -> Iterator[ValidationError]:
        pass

class Draft7Validator:
    def __init__(
        self,
        schema: _SchemaT | str,
        formats: dict[str, _FormatFunc] | None = None,
    ) -> None:
        pass

    def is_valid(self, instance: Any) -> bool:
        pass

    def validate(self, instance: Any) -> None:
        pass

    def iter_errors(self, instance: Any) -> Iterator[ValidationError]:
        pass

class Draft201909Validator:
    def __init__(
        self,
        schema: _SchemaT | str,
        formats: dict[str, _FormatFunc] | None = None,
    ) -> None:
        pass

    def is_valid(self, instance: Any) -> bool:
        pass

    def validate(self, instance: Any) -> None:
        pass

    def iter_errors(self, instance: Any) -> Iterator[ValidationError]:
        pass

class Draft202012Validator:
    def __init__(
        self,
        schema: _SchemaT | str,
        formats: dict[str, _FormatFunc] | None = None,
    ) -> None:
        pass

    def is_valid(self, instance: Any) -> bool:
        pass

    def validate(self, instance: Any) -> None:
        pass

    def iter_errors(self, instance: Any) -> Iterator[ValidationError]:
        pass

def validator_for(
    schema: _SchemaT,
    formats: dict[str, _FormatFunc] | None = None,
) -> Draft4Validator | Draft6Validator | Draft7Validator | Draft201909Validator | Draft202012Validator:
    pass
