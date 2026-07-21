// support: Prompt, PromptEncode, Trusted, Untrusted, Public, Sanitized, Secret, PromptInjectionRisk
// layer: effect
// polarity: positive
// status: covered-positive
module tests.compiler.std_conformance.effect.prompt_trust_secret_allowed_paths;

type Incident = { id: string, body: string };

flow redact(incident: Secret<Incident>) -> Sanitized<string> {
    return Sanitized("redacted");
}

flow build_prompt(instruction: Trusted<string>, input: Sanitized<string>) -> Prompt {
    return Prompt.new()
        .system(instruction)
        .data(input)
        .user(Public("summarize"));
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
