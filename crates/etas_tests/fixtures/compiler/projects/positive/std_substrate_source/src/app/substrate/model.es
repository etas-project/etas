module app.substrate.model;

public type SubstrateReport = {
    name: string,
    count: i32,
};

public flow build_report() -> SubstrateReport ![]
{
    return SubstrateReport { name = "std_substrate", count = 4 };
}
