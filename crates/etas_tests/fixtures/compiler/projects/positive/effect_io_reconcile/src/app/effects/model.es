module app.effects.model;

public type EffectReport = {
    name: string,
    count: i32,
};

public flow build_report(seed: i32) -> EffectReport ![]
{
    return EffectReport { name = "io_reconcile", count = seed + 1 };
}
