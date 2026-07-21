module app.knowledge.index;

import std.collections.Array;

public type Article = {
    title: string,
    body: string,
};

public type KnowledgeIndex = {
    articles: Array<Article>,
};

public flow sample_index() -> KnowledgeIndex ![] {
    return KnowledgeIndex {
        articles = [
            Article { title = "Onboarding", body = "Start here" },
        ],
    };
}
