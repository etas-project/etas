module app.effects.model;

public type EffectInput = {
    body: string,
};

public flow sample_input() -> EffectInput ![]
{
    return EffectInput { body = "negative project effect" };
}
