module session_handoff;

import std.agent.session.{SessionLease, with_lease, handoff};
import std.io.println;

enum Department { Billing, Support }
effect Routing {
    action handoff(department: Department, text: string) -> never;
}

@model("configured-model")
agent Triage = {};
impl Triage {
    public flow classify(self, text: string) -> never {
        let department = perform infer<Department>(text,
            system = Trusted("Route the ticket to Billing or Support."));
        perform Routing.handoff(department, text);
    }
}

spec SupportDesk ~ std.agent.Active {
    flow take(self, lease: SessionLease, text: string) -> string ![];
}

agent BillingDesk = {};
impl BillingDesk ~ SupportDesk {
    public flow take(self, lease: SessionLease, text: string) -> string ![] {
        return "Billing received: " + text;
    }
}

agent TechnicalDesk = {};
impl TechnicalDesk ~ SupportDesk {
    public flow take(self, lease: SessionLease, text: string) -> string ![] {
        return "Support received: " + text;
    }
}

flow main(args: Array<string>) -> i32 {
    let config = SessionConfig.continue_or_new("handoff-demo");
    let triage = Triage {};
    let billing = BillingDesk {};
    let support = TechnicalDesk {};
    // Classification is mocked; lease acquisition, handoff and cleanup are real.
    let reply = Route(config, triage, billing, support, "Please explain my invoice.") with {
        Agentic.infer<_, Department>(_, _) => Department.Billing;
    };
    println(reply);
    if reply != "Billing received: Please explain my invoice." { return 1; }
    return 0;
}

flow Route(config: SessionConfig, triage: Triage, billing: BillingDesk,
    support: TechnicalDesk, text: string) -> string {
    return with_lease(config, lease =>
        handle triage.classify(text) with {
            Routing.handoff(department, message) => {
                finish handoff(lease, next => match department {
                    Department.Billing => billing.take(next, message),
                    Department.Support => support.take(next, message),
                });
            }
        }
    );
}
