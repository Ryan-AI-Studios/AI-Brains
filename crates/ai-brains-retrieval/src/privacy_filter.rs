pub fn is_injectable_privacy(privacy: &str) -> bool {
    privacy != "\"Sealed\""
        && privacy != "\"NeverInject\""
        && privacy != "\"Never Inject\""
        && privacy != "\"Private\""
}
