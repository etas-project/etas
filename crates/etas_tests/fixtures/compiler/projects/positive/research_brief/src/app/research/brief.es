module app.research.brief;

import app.research.model.{Brief, Citation};

public flow make_brief(topic: string) -> Brief ![] {
    return Brief {
        title = topic,
        citations = [
            Citation { title = "source", url = "https://example.invalid" },
        ],
    };
}
