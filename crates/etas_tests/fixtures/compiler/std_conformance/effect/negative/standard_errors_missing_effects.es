// support: ValidationError, ToolTimeout, ToolDenied, PolicyDenied, EffectBoundaryViolation, SandboxViolation, PromptInjectionRisk, MissingCitation, ProtocolViolation, HumanRejected, IndexError, Error.raise
// layer: effect
// polarity: negative
// status: blocked-by-impl
// expect: every standard error raise must appear in the effect signature or be handled
module tests.compiler.std_conformance.effect.standard_errors_missing_effects;

flow missing_validation() -> never {
    Error<ValidationError>.raise(ValidationError.InvalidValue);
}

flow missing_tool_timeout() -> never {
    Error<ToolTimeout>.raise(ToolTimeout.Expired);
}

flow missing_tool_denied() -> never {
    Error<ToolDenied>.raise(ToolDenied.NotAllowed);
}

flow missing_policy_denied() -> never {
    Error<PolicyDenied>.raise(PolicyDenied.Blocked);
}

flow missing_effect_boundary() -> never {
    Error<EffectBoundaryViolation>.raise(EffectBoundaryViolation.OutsideBoundary);
}

flow missing_sandbox() -> never {
    Error<SandboxViolation>.raise(SandboxViolation.EscapeAttempt);
}

flow missing_prompt_risk() -> never {
    Error<PromptInjectionRisk>.raise(PromptInjectionRisk.UntrustedControlText);
}

flow missing_citation() -> never {
    Error<MissingCitation>.raise(MissingCitation.Required);
}

flow missing_protocol() -> never {
    Error<ProtocolViolation>.raise(ProtocolViolation.UnexpectedMessage);
}

flow missing_human_rejected() -> never {
    Error<HumanRejected>.raise(HumanRejected.Denied);
}

flow missing_index() -> never {
    Error<IndexError>.raise(IndexError.OutOfBounds);
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
