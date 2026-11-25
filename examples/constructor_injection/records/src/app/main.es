module app.main;
import wiring.library.wiring.{Supply, Settings, Box, IntBox, NamedBox, NumberKey};
flow main(args: Array<string>) -> i32 {
    var order = "";
    var calls = 0;
    let settings = handle {
        Settings {
            explicit = { order = order + "E"; 9 },
        }
    } with {
        Supply.first() => {
            calls = calls + 1;
            if calls == 1 { order = order + "A"; }
            else { order = order + "B"; }
            resume calls;
        }
    };

    let first = 11;
    let second = 22;
    let explicit = 33;
    let supplied = Settings { second, first = first, explicit };
    let named = handle {
        NamedBox { marker = 3 }
    } with {
        Supply.value<i32>() => { resume 17; }
    };
    let number = handle {
        Box { marker = 3 }
    } with {
        NumberKey => { 7 };
    };
    let text = handle {
        Box { marker = "marker" }
    } with {
        string => "injected";
    };
    let alias_value = handle {
        IntBox { marker = 8 }
    } with {
        i32 => 9;
    };

    if order == "EAB" && calls == 2
        && settings.first == 1 && settings.second == 2 && settings.explicit == 9
        && supplied.first == 11 && supplied.second == 22
        && named.value == 17 && number.value == 7 && text.value == "injected" && alias_value.value == 9 {
        return 0;
    }
    return 1;
}
