module app.effects.model;

public type EffectReport = {
    name: string,
    count: i32,
};

public flow build_report(seed: i32) -> EffectReport ![]
{
    return EffectReport { name = "console_metrics", count = seed + 1 };
}
