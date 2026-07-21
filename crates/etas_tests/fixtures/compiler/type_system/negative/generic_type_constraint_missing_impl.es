module tests.compiler.type_system.negative.generic_type_constraint_missing_impl;

spec Region;
spec Within<Root>;

type ReportsRoot;
type ArchiveRoot;
type WorkspacePath<R>;

impl ArchiveRoot ~ Region;

flow archive_path() -> WorkspacePath<ArchiveRoot>
{
    return abort("path value is provided by the host in runtime tests");
}

flow read_report<R ~ Region + Within<ReportsRoot>>(
    path: WorkspacePath<R>
) -> bytes
{
    return abort("read_report is type-system only");
}

flow main(args: Array<string>) -> i32
{
    let result: bytes = read_report<ArchiveRoot>(archive_path());
    return 0;
}
