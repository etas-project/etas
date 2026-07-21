module tests.compiler.type_system.positive.generic_type_constraints;

spec Region;
spec Within<Root>;
spec RegionWithin<R, Root> ~ Region + Within<R> + Within<Root>;

type WorkspaceRoot;
type ReportsRoot;
type ArchiveRoot;
type WorkspacePath<R>;

impl ReportsRoot ~ RegionWithin<ReportsRoot, WorkspaceRoot>;

flow reports_path() -> WorkspacePath<ReportsRoot>
{
    return abort("path value is provided by the host in runtime tests");
}

flow read_under<R ~ RegionWithin<R, WorkspaceRoot>>(
    path: WorkspacePath<R>
) -> bytes
{
    return abort("read_under is type-system only");
}

flow read_report<R ~ Region + Within<ReportsRoot>>(
    path: WorkspacePath<R>
) -> bytes
{
    return abort("read_report is type-system only");
}

flow main(args: Array<string>) -> i32
{
    let first: bytes = read_under<ReportsRoot>(reports_path());
    let second: bytes = read_report<ReportsRoot>(reports_path());
    return 0;
}
