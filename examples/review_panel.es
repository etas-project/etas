module review_panel;

import std.io.println;

effect Review {
    action check(draft: string) -> bool;
}

@model("configured-model")
agent Writer = {};
impl Writer {
    public flow write(self, topic: string) -> Option<string> {
        var draft = "";
        for _ in [0, 1, 2] {
            draft = perform infer<string>(Prompt.new()
                .system(Trusted("Write or revise the draft for review."))
                .data(topic).data(draft));
            if perform Review.check(draft) { return Some(draft); }
        }
        return None;
    }
}

spec Reviewer ~ std.agent.Active {
    flow review(self, draft: string) -> bool ![];
}

flow Panel(writer: Writer, reviewers: Array<RuleReviewer>, topic: string) -> Option<string> {
    return writer.write(topic) with {
        Review.check(draft) => {
            let votes = std.task.map_concurrent(reviewers, r => r.review(draft));
            !votes.is_empty() && votes.iter().all(vote => vote)
        }
    };
}

flow TestReview(writer: Writer, topic: string) -> Option<string> {
    return writer.write(topic) with {
        Review.check(draft) => !std.text.is_empty(draft);
    };
}

enum ReviewRule { Length, Topic }
agent RuleReviewer = { rule: ReviewRule };
impl RuleReviewer ~ Reviewer {
    public flow review(self, draft: string) -> bool ![] {
        return match self.rule {
            ReviewRule.Length => std.text.len(draft) > 20,
            ReviewRule.Topic => std.text.contains(draft, "Etas"),
        };
    }
}

flow main(args: Array<string>) -> i32 {
    let writer = Writer {};
    let reviewers = [
        RuleReviewer { rule = ReviewRule.Length },
        RuleReviewer { rule = ReviewRule.Topic },
    ];
    // Model results are fixtures; the panel still executes real agent methods.
    let reviewed = Panel(writer, reviewers, "Introduce Etas") with {
        Agentic.infer<_, string>(_, _) => "Etas coordinates agents with typed effects and handlers.";
    };
    match reviewed {
        Some(draft) => println(draft),
        None => { return 1; },
    };
    let exhausted = Panel(writer, reviewers, "Introduce Etas") with {
        Agentic.infer<_, string>(_, _) => "Too short";
    };
    match exhausted {
        None => println("No approved draft after three attempts."),
        Some(_) => { return 1; },
    };
    return 0;
}
