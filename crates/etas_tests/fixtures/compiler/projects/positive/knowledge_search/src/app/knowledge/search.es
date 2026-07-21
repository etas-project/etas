module app.knowledge.search;

import app.knowledge.index.{Article, KnowledgeIndex};

public flow search(index: KnowledgeIndex, query: string) -> Article ![] {
    return Article {
        title = query,
        body = "matched document",
    };
}
