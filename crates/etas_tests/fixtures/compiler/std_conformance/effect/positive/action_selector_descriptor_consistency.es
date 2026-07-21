// support: ActionSelectorDescriptor, Workspace.read, Browser.navigate, Email.send
// layer: effect
// polarity: positive
// status: covered-positive
module tests.compiler.std_conformance.effect.action_selector_descriptor_consistency;

spec WorkspaceRegion;
spec BrowserDomain;
spec DeliverableAddress;

type ReportsRoot;
type DocsDomain;
type WorkAccount;
type WorkspacePath = string;
type BrowserUrl = string;
type EmailDraft = {
    subject: string,
    body: string,
};

impl ReportsRoot ~ WorkspaceRegion;
impl DocsDomain ~ BrowserDomain;
impl WorkAccount ~ DeliverableAddress;

effect Workspace {
    action read<R ~ WorkspaceRegion>(path: WorkspacePath) -> bytes;
}

effect Browser {
    action navigate<D ~ BrowserDomain>(url: BrowserUrl) -> unit;
}

effect Email {
    action send<A ~ DeliverableAddress>(draft: EmailDraft) -> unit;
}

flow read_workspace(path: WorkspacePath) -> bytes ![Workspace.read<ReportsRoot>]
{
    return perform Workspace.read<ReportsRoot>(path);
}

flow open_docs(url: BrowserUrl) -> unit ![Browser.navigate<DocsDomain>]
{
    perform Browser.navigate<DocsDomain>(url);
    return;
}

flow send_work(draft: EmailDraft) -> unit ![Email.send<WorkAccount>]
{
    perform Email.send<WorkAccount>(draft);
    return;
}

flow main(args: Array<string>) -> i32
{
    return 0;
}
