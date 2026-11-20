# Hippogrove

Ein mobiles Cozy-3D-Götterspiel in Rust und Bevy. Als junger Nilpferdgott hilfst du einem Dorf am Fluss, baust es aus und gewinnst Glauben. Deine Wolkenpfote greift direkt in die Welt ein. Ein aufrecht laufendes Nilpferd sammelt, trägt und lernt an deiner Seite.

Hippogrove 1.0 spielt sich im Querformat per Touch oder mit angeschlossenem Controller. Vier Welten führen vom Weidental über die Kiefernbucht und die Reisterrassen bis ans Papyrusufer. Baue Dörfer am Fluss aus und gewinne jedes Mal drei Nachbarsiedlungen durch Versorgung und Ansehen oder durch Wunder und Kreaturenwachen. Talente und Kreaturenrang reisen mit. Alle Städte, Straßen und Entscheidungen bleiben im lokalen Spielstand erhalten.

![Bevy-Spielaufnahme aus Hippogrove 1.0 mit Wegen, Nilpferd und Wolkenpfote](docs/weidental-v1-spiel.jpg)

## Spielen

Auf Android öffnest du die signierte Test-APK aus deinen Downloads. Benötigt werden Android 8 oder neuer, ein 64-Bit-Gerät und Vulkan. [Android-Testversionen](https://github.com/bthginfo/Hippogrove/releases)

Tippe auf einen Holzstoß und bestätige **Greifen**. Wähle dann den Sammelplatz und **Ablegen**. Mit **Gläubige** und **Nilpferd** erteilst du Sammelaufträge. **Zum Ziel** zeigt einen passenden Zielpunkt für das aktive Werkzeug. In **Fortschritt** setzt du Talentpunkte ein. Das Nilpferd lernt durch Lob nach einer Lieferung, Wache oder Ernte.

Unter **Planen** zeichnest du Wege mit Anfang und Ende. Ausbauten brauchen eine Werkstatt und Anschluss an einen Dorfweg. Am Wegenetz bewässern Brunnen die Felder, Märkte tauschen Vorräte und Wachtürme schützen den Umkreis. Unter **Bauen** wechselst du zwischen Grundbau, Versorgung und Grenze. Auf der **Landkarte** springst du zwischen Startdorf, Übergang und den fernen Siedlungen.

Tippe auf den Titel des aktiven Auftrags, um ihn einzuklappen. Der goldene Knopf zeigt die Aktion für das ausgewählte Ziel. Im Nilpferd-Werkzeug findest du Lob, Snack und Pause unter **Pflege**. Der Fokusrahmen zeigt im Menü, welche Aktion du per Controller oder Tastatur bestätigst.

| Steuerung | Kamera und Ziel | Aktionen |
| --- | --- | --- |
| Touch | Boden ziehen, mit zwei Fingern schwenken und zoomen, Ziel antippen | Werkzeugleiste und Bestätigung |
| Controller | Linker Stick bewegt die Wolkenpfote, rechter Stick schwenkt, Trigger zoomen | Steuerkreuz wählt Werkzeuge oder Menüpunkte, Südtaste bestätigt, Osttaste bricht ab, Start öffnet die Karte |
| Rechner | Maus zum Antippen, Ziehen und Zoomen, WASD zum Schwenken | 1–6 für Werkzeuge, E für die Aktion, R zum Drehen, G fürs Ziel, M für die Landkarte, Tab für Kapitel, Q für Aufträge, P für Fortschritt. Pfeiltasten und Enter bedienen Menüs. F12 speichert ein Spielbild. |

## Projekt bauen

```sh
cargo run --locked
cargo test --locked --lib
```

Unter Linux braucht die Desktop-Entwicklung X11, Vulkan und `libudev-dev`. `HIPPOGROVE_SAVE_DIR` kann für einen getrennten Testspielstand auf ein anderes Verzeichnis zeigen.

### Android

Die Paketierung verwendet JDK 17 oder neuer, Python 3, Android SDK Platform 36, Build Tools 36.0.0 und NDK r28c. Setze `ANDROID_HOME` auf das SDK.

```sh
rustup target add aarch64-linux-android x86_64-linux-android
tools/build-android.sh
```

Die signierte Testdatei liegt unter `build/android/`. Für spätere Updates der Testversion wird derselbe Schlüssel aus `build/android/hippogrove-test.jks` verwendet.

### iOS und iPadOS

Das Paketierungsskript benötigt einen Mac mit Xcode. Auf Apple Silicon ist das Rust-Ziel `aarch64-apple-ios-sim` für den Simulator vorgesehen. Auf einem Intel-Mac ist es `x86_64-apple-ios`.

```sh
rustup target add aarch64-apple-ios-sim
tools/build-ios.sh
```

Ein Gerätebuild mit `HIPPOGROVE_IOS_DEVICE=1` verwendet `aarch64-apple-ios` und braucht für die Installation eine Apple-Entwicklungssignatur mit passendem Provisioning-Profil.

## Referenzen und Modelle

Black & White und Black & White 2 liefern die Spielideen für Hand, Einfluss, Kreatur und Siedlungsbau. Waldwinkel ist die Vorlage für die Low-Poly-Optik, die Bedienung und die mobile Paketierung. Der ausführliche Plan enthält weitere Spielreferenzen und die geprüften Skills. [Waldwinkel](https://github.com/bthginfo/waldwinkel) · [Entwicklungsplan](PLAN.md)

Die Blender-Quelle liegt in `art/Hippogrove.blend`. `blender -b -P art/build_scene.py` erzeugt das Nilpferd und seinen GLB-Export. In Bevy bewegen sich Arme und Beine über getrennte Drehpunkte. Die Referenzen für 1.0 lagen vor dem Umbau vor. [Weidental](docs/weidental-uebersicht-v1.png) · [Stadtplanung](docs/weidental-stadtplanung-v1.png) · [Kiefernbucht, Reisterrassen und Papyrusufer](docs/regionen-v1-referenz.png)

## Geprüfter Stand

Die Rust-Tests spielen den Friedensweg durch alle vier Kapitel und den Kampfweg im Weidental. Sie prüfen Wege und Dienste, Ausbauten, Einfluss, Kreaturentraining, Wunder und das Laden älterer Spielstände. Änderungen, Spielaufnahmen und Plattformprüfungen stehen in den Versionsnotizen. [Entwicklung und Prüfungen für 1.0](docs/V1.md)
