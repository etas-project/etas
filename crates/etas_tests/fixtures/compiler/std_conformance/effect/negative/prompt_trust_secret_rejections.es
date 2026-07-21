// support: Prompt, PromptEncode, Trusted, Untrusted, Public, Sanitized, Secret, PromptInjectionRisk
// layer: effect
// polarity: negative
// status: covered-negative
// expect: Secret<T> is not prompt-encodable by default
// expect: Prompt.system requires trusted instruction content
// expect: Sanitized<T> does not automatically become Trusted<T>
module tests.compiler.std_conformance.effect.prompt_trust_secret_rejections;

type Incident = { id: string, body: string };

flow leak_secret(secret: Secret<Incident>) -> Prompt {
    return Prompt.new().data(secret);
}

flow untrusted_system(raw: Untrusted<string>) -> Prompt {
    return Prompt.new().system(raw);
}

flow sanitized_system(clean: Sanitized<string>) -> Prompt {
    return Prompt.new().system(clean);
}

agent BadWriter(input: Secret<Incident>) -> string {
    return Prompt.new().system(input);
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
