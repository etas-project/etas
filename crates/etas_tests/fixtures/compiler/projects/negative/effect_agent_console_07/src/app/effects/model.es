module app.effects.model;

public type EffectInput = {
    body: string,
};

public type EffectOutput = {
    summary: string,
    score: i32,
};

public flow sample_input() -> EffectInput ![]
{
    return EffectInput { body = "negative project effect" };
}
