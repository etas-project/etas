module app.dependency_bridge;

public import dep.math.*;

public flow dependency_ready() -> bool ![] {
    return true;
}
