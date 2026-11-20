# Hippogrove

Ein mobiles Cozy-3D-Götterspiel in Rust und Bevy. Als junger Nilpferdgott hilfst du einem Dorf am Fluss, baust es aus und gewinnst Glauben. Deine Wolkenpfote greift direkt in die Welt ein. Ein aufrecht laufendes Nilpferd sammelt, trägt und lernt an deiner Seite.

Hippogrove spielt sich im Querformat per Touch oder mit angeschlossenem Controller. Fünf Themenkampagnen haben je drei Kapitel. In Europa, im nordischen Fjord, auf ostasiatischen Terrassen, am afrikanisch inspirierten See und in der griechischen Ägäis stiftest du Bündnisse oder gewinnst Siedlungen mit Wundern und Wachen. Für jede Landschaft gibt es auch **Freies Bauen** mit offenen Brücken, allen Plänen und einem eigenen Spielstand. [Kampagnen und Sandbox](docs/V1.md#kampagnen-und-freies-bauen)

![Der Ägäisbogen im Spiel mit Theater, Hafen, Badhaus und dem Nilpferd](docs/aegaeisbogen-spiel.jpg)

## Spielen

Auf Android öffnest du die signierte Test-APK aus deinen Downloads. Benötigt werden Android 8 oder neuer, ein 64-Bit-Gerät und Vulkan. [Hippogrove 1.1.1 für Android](https://github.com/bthginfo/Hippogrove/releases/tag/v1.1.1)

Beim ersten Start landest du auf der **Übungsinsel**. Der Marker zeigt dir Holz, Sammelplatz, Bauplatz, Feld und Nilpferd. Tippe auf **Greifen**, um bis zu sechs Einheiten aufzuheben, oder halte den Knopf für bis zu 18. Lass los, wenn die gewünschte Menge angezeigt wird. **Zum Ziel** führt zum nächsten offenen Auftrag und wählt das passende Werkzeug. Nach der Übung öffnet sich die Kampagnenauswahl. Du kannst die Insel später über die Spielmodi wieder besuchen.

In einem neuen Weidental entscheidet die erste Holzladung, wem du zuerst hilfst. Mira nutzt sechs Holz für das erste Dach im Dorf. Jora repariert damit ihren Hof am anderen Ufer und schickt Stein und Nahrung zurück. Mit einem Bündel in der Pfote leuchten die Ablageplätze, an denen du es abgeben kannst.

Unter **Wege** zeichnest du Wege mit Anfang und Ende. Ausbauten brauchen eine Werkstatt und einen Anschluss an den Dorfweg. Das **Baubuch** zeigt 33 Gebäudearten pro Landschaft, darunter das automatisch öffnende Palisadentor und einen regionalen Bau. Insgesamt gibt es 37 Gebäudearten. Im **Forschungsmenü** erschließt du 12 Technologien über vier Zweige. Am Wegenetz können Familien wachsen, Betriebe Vorprodukte und Werkzeug herstellen, Treffpunkte die Stimmung heben und Wachhäuser Wachen ausbilden. Auf der **Landkarte** springst du zu Dörfern und Rohstoffgebieten. Dort baust du Sammellager, die Wege verkürzen und deiner Pfote den Abbau vor Ort erlauben. Unter **Vorräte** siehst du die Rohstoffe, Vorprodukte und das getragene Bündel als bebilderte Gegenstände.

Wähle auf der **Landkarte** ein Rohstoffgebiet, um seine drei Vorkommen zu vergleichen. Mit einem Sammellager kannst du dort Gläubige und das Nilpferd beauftragen. Sie wechseln zu einer anderen erreichbaren Quelle, wenn die erste leer ist. **Zur Quelle** richtet die Kamera für den Abbau von Hand aus. Mit einer Ladung in der Pfote führt **Zum Ziel** zum nächsten Lagerplatz. [Sammelaufträge und Bildvorlage](docs/Sammelauftraege.md)

Die fünf Kampagnen haben je drei Inselkarten mit eigenen Küsten, Höhenzügen, Übergängen und Startplätzen. Im Weidenland ziehen sich Waldarme um die Mühlen. Der Fjord hat lang gestreckte Landzungen, die Terrassen große Bewässerungsbecken, das Seeufer Sandbuchten und die Ägäis vorgelagerte Felsinseln. Neue Spielstände und erstmals betretene Kapitel nutzen diese Landschaften. Beim Fortsetzen lädt jedes schon besuchte Kapitel die Karte seines Spielstands.

Das Nilpferd wächst durch Lieferungen, Ernten und erfolgreiche Wachen. Wähle es mit der Pfote, um es zu streicheln. Lobe es nach einer Holzlieferung. Wenn es später frei ist und acht Holz oder weniger im Lager liegen, holt es selbst Nachschub. **Pause** unterbricht die Hilfe. **Sammeln** oder ein neues Lob setzt sie fort. Unter **Fortschritt** siehst du seine nächste Wachstumsstufe. Hilfe und Bündnisse bringen der Stadt mehr Glauben und Stimmung. Einschüchterung stärkt den Schutz durch Wachtürme und kostet Stimmung. Stadt und Kreatur zeigen ihre Ausrichtung auch im Spielbild.

Auf den neuen Inseln bringen Rivale und Kreatur eine eigene Bedrohung mit. Ivens Mooshirsch beansprucht Holz, Sigruns Runenwolf bringt Frost, Lins Wolkendrache verzögert die Ernte, Nurus Schilfschlange vertreibt Sammler und Nereas Windgreif zerstreut Wunderkraft. Die Aufträge zeigen den nächsten Angriff und mögliche Gegenmaßnahmen. Du kannst die Nachbarn mit einem Bündnis gewinnen oder ihre Wachen und Kreatur mit Wundern zurückdrängen.

Jedes Kapitel hat eigene Bitten und einen Nebenauftrag. Im Weidenland versorgst du Mühlen und Höfe, im Fjord sicherst du Winterquartiere und auf den Terrassen bringst du mehrere Reisernten ein. Am See verbinden neue Wege die Brunnen und Märkte. In der Ägäis produzierst du Öl und richtest ein Theater für das Fest der Winde ein. Das Auftragsfenster zeigt den Fortschritt jedes Ziels. Schon besuchte Kapitel aus älteren Spielständen behalten ihre Aufträge.

Im Entwicklungsstand 1.2 führt dich der nächste Auftragsschritt oben rechts zu den Details. Ein `!` auf der Karte kennzeichnet einen verfügbaren Nebenauftrag, ein `?` eine abholbare Belohnung. Der goldene Knopf zeigt die Aktion am ausgewählten Ziel. **Nilpferd** öffnet die Befehle zum Sammeln, Herbeirufen und zur Pflege. Das **Zauberbuch** zeigt die Voraussetzungen für Wunder. Der Fokusrahmen zeigt im Menü, welche Aktion du per Controller oder Tastatur bestätigst.

| Steuerung | Kamera und Ziel | Aktionen |
| --- | --- | --- |
| Touch | Boden ziehen, mit zwei Fingern schwenken, mit der Geste oder **+** und **−** zoomen, Ziel antippen | Werkzeugleiste und Bestätigung |
| Controller | Linker Stick bewegt den Zielpunkt, rechter Stick schwenkt, Trigger zoomen | Links und rechts wechseln das Werkzeug, oben zeigt den Auftrag. R1 zeigt das Ziel des gewählten Werkzeugs, unten und L1 öffnen die Werkzeugoptionen. In Menüs wählst du mit Stick oder Steuerkreuz, bestätigst mit A/Kreuz und gehst mit B/Kreis zurück. Start öffnet das Menü, Select oder Options die Landkarte. |
| Rechner | Maus zum Antippen und Ziehen, Mausrad oder **+** und **−** zum Zoomen, WASD zum Schwenken | 1–6 für Werkzeuge, E für die Aktion oder zum Halten beim Greifen, R zum Drehen, G für das Auftragsziel, H für das Werkzeugziel, M für die Landkarte, Tab für Kapitel, Q für Aufträge, P für Fortschritt, B fürs Baubuch und T für Forschung, I für Vorräte und F für Spielmodi. Pfeiltasten und Enter bedienen Menüs. F12 speichert ein Spielbild. |

## Projekt bauen

```sh
cargo run --locked
cargo test --locked --lib
```

Unter Linux braucht die Desktop-Entwicklung X11, Vulkan, `libudev-dev` und `libasound2-dev`. `HIPPOGROVE_SAVE_DIR` kann für einen getrennten Testspielstand auf ein anderes Verzeichnis zeigen.

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

Die Blender-Quelle liegt in `art/Hippogrove.blend`. `blender -b -P art/build_scene.py` erzeugt das Nilpferd und seinen GLB-Export. In Bevy bewegen sich Arme und Beine über getrennte Drehpunkte. Für die neuen Gebäude entstanden eigene Low-Poly-Modelle und Materialien. Die Referenzen lagen vor dem Modellieren vor. [Modellquellen der Gebäude](src/art/settlement_models.rs) · [Gebäudestudie](docs/gebaeude-referenz.png) · [Ägäisbogen](docs/aegaeisbogen-referenz.png) · [Forschungsoberfläche](docs/forschung-referenz.png)

`blender -b -P art/build_villages.py` erzeugt die Hütten und Lager aller fünf Welten. Jedes der Gebäude ist ein einzelnes Mesh mit eingearbeiteten Farben und Reliefdetails. [Blender-Quelle für die Dörfer](art/build_villages.py)

Die Karten reichen bis 178 × 140 Felder. Gebirgsketten, Täler, dichte Haine, Felsformationen und modellierte Küsten machen die Landschaft lesbar. Beim Sammeln kippen Bäume, Stein bricht in Splitter und Früchte lösen sich vom Strauch. Bevy stimmt Sonnenwinkel, Fülllicht, Dunst und Schatten auf die jeweilige Welt ab. Das Höhenprofil bestimmt gleichzeitig, wo Figuren laufen, wo Regen auftrifft und welchen Bodenpunkt die Wolkenpfote auswählt. Acht Vegetations-, Siedlungs- und Rohstoffmodelle stammen aus Waldwinkel 1.1.0. [Licht, Wetter und Herkunft der Modelle](docs/V1.md#gelände-licht-wetter-und-physik)

Die Hafenlaterne, der Wachfarn und die Umgebungs- und Aktionstöne stammen aus Waldwinkel 1.5. Die Instrumentalspur „Between the Olives“ wird mit `art/build_music.py` erzeugt. Musik, Umgebung und Effekte lassen sich im Spielmenü unter **Ton** einzeln schalten. [Waldwinkel 1.5](https://github.com/bthginfo/waldwinkel/tree/8a98a76a2921690b533015e3f57fadccc2c66b24)

## Geprüfter Stand

Die Rust-Tests spielen den Friedensweg durch die 15 Themenkapitel sowie die fünf Kapitel der Chronik und den Kampfweg im Weidental. Sie prüfen auch die Forschungs- und Produktionsketten, Geburten, Patrouillen, Wunder, die Menüführung und die Übernahme älterer Spielstände. [Spielaufnahmen und Prüfungen für 1.0](docs/V1.md) · [Stadtentwicklung](docs/Stadtentwicklung.md)
