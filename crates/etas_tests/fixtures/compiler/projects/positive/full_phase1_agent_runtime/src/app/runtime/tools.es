module app.runtime.tools;

import app.runtime.model.{RuntimeRequest};

public tool EvidenceLookup(input: RuntimeRequest) -> string {
    return "local evidence for " + input.topic;
}
