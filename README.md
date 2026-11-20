# Hippogrove

Ein mobiles Cozy-3D-Götterspiel in Rust und Bevy. Als junger Nilpferdgott hilfst du einem Dorf am Fluss, baust es aus und gewinnst Glauben. Deine Wolkenpfote greift direkt in die Welt ein. Ein aufrecht laufendes Nilpferd sammelt, trägt und lernt an deiner Seite.

**[Hippogrove 1.4.0 für Android herunterladen](https://raw.githubusercontent.com/bthginfo/Hippogrove/ad64d5e6512126c3d96f0d13012b6908bec09daf/Hippogrove-1.4.0.apk)** · [Release-Notizen](https://github.com/bthginfo/Hippogrove/releases/tag/v1.4.0) · [SHA-256](https://raw.githubusercontent.com/bthginfo/Hippogrove/ad64d5e6512126c3d96f0d13012b6908bec09daf/Hippogrove-1.4.0.apk.sha256)

Version 1.4.0 verbindet Kreaturentraining, Dorfproduktion und die Reise zur nächsten Insel. Lobe eine Lieferung, bring Baumaterial direkt zum Übergang und entscheide, wie du die Nachbardörfer gewinnst. Bestehende Standalone-Spielstände lassen sich fortsetzen.

Die signierte Standalone-App braucht Android 8 oder neuer und Vulkan. Die APK unterstützt ARM64 und x86_64. Die frühere Playtest-App und die Standalone-App haben eigene Spielstände.

Hippogrove spielt sich im Querformat per Touch oder mit angeschlossenem Controller. Fünf Themenkampagnen haben je drei Kapitel. In Europa, im nordischen Fjord, auf ostasiatischen Terrassen, am afrikanisch inspirierten See und in der griechischen Ägäis stiftest du Bündnisse oder gewinnst Siedlungen mit Wundern und Wachen. Für jede Landschaft gibt es auch **Freies Bauen** mit offenen Brücken, allen Plänen und einem eigenen Spielstand. [Kampagnen und Sandbox](docs/V1.md#kampagnen-und-freies-bauen)

![Der Ägäisbogen im Spiel mit Theater, Hafen, Badhaus und dem Nilpferd](docs/aegaeisbogen-spiel.jpg)

## Spielen

Öffne die heruntergeladene APK auf deinem Android-Gerät und installiere sie. In der App-Übersicht findest du das Spiel unter **Hippogrove**. [Alle Veröffentlichungen](https://github.com/bthginfo/Hippogrove/releases)

Wähle im Hauptmenü **Tutorial**. Auf der Übungsinsel führen acht Etappen vom ersten Holzbündel bis zur Ernte und zur Pflege deines Nilpferds. Ein Marker bleibt an deinem Ziel. Du öffnest das Baurad, wählst selbst eine Hütte und bestätigst erst danach den Bau. Nach der Übung kannst du direkt ins Weidental reisen oder eine andere Kampagne wählen. Der erste Auftrag wartet, bis du **Zum ersten Ziel** wählst. Über **Menü → Hauptmenü** kannst du die Partie verlassen und später mit **Spiel fortsetzen** wieder einsteigen.

Tippe in der Welt auf ein Objekt oder drücke am Controller **R3**. Am Holzstoß kannst du ein Bündel greifen, dein Nilpferd losschicken oder einen freien Dorfbewohner einteilen. Ein fast volles Lager nimmt an, was hineinpasst. Der Rest bleibt in deiner Pfote. Halte direkt auf dem Holz oder auf **Greifen**, um bis zu 18 Einheiten zu nehmen. Ein trockenes Feld bietet Regen an, ein reifes Feld die Ernte. Bei einer Familie oder Hütte kannst du einmal pro Spieltag drei Nahrung teilen. Das bringt zwei Glauben und hebt die Stimmung des Dorfes für diesen Tag. Kosten und Erträge erscheinen am Ort der Aktion.

![Objektmenü am Holzstoß mit den Aktionen Bündel greifen und Nilpferd sammeln lassen](docs/objektmenue-weidental.jpg)

Im Hauptmenü erreichst du unter **Ton** den **Testton** und kannst Musik, Umgebung und Effekte einzeln schalten oder gemeinsam einschalten. Der Testton nutzt die Medienlautstärke deines Geräts.

In einem neuen Weidental entscheidet die erste Holzladung, wem du zuerst hilfst. Mira nutzt sechs Holz für das erste Dach im Dorf. Jora repariert damit ihren Hof am anderen Ufer und schickt Stein und Nahrung zurück. Mit einem Bündel in der Pfote leuchten die Ablageplätze, an denen du es abgeben kannst.

Das **Baubuch** zeigt zu jedem Bauplan das Gebäude in seinem Landschaftsstil, seinen Zweck und die benötigten Rohstoffe. Es gibt 33 Gebäudearten pro Landschaft und 37 insgesamt. Wähle einen Bauplan, um den Grundriss in der Welt zu sehen. Die Vorschau zeigt, ob der Bauplatz frei ist oder was noch fehlt. **Freier Platz** sucht eine Fläche in der Nähe. Erst **Bauen** verbraucht Material.

Wähle ein errichtetes Gebäude oder öffne **Wege → Ausbau**, um Stufe 2 anzusehen. Das Fenster zeigt den Nutzen, den Materialpreis und die nötigen Gläubigen und den Glauben. In Kampagnen brauchst du zusätzlich eine Werkstatt und einen Dorfweg zum Gebäude. Glauben wächst unter anderem mit neuen Gebäuden, Ernten und erfüllten Aufträgen. Wunder verbrauchen Wunderkraft, die sich wieder auflädt. Beide Werte stehen auch unter **Vorräte**.

Unter **Wege** zeichnest du Wege mit Anfang und Ende. Im **Forschungsmenü** erschließt du zwölf Technologien über vier Zweige. Am Wegenetz können Familien wachsen, Betriebe Vorprodukte und Werkzeug herstellen, Treffpunkte die Stimmung heben und Wachhäuser Wachen ausbilden. Auf der **Landkarte** springst du zu Dörfern und Rohstoffgebieten und baust Sammellager.

Wähle auf der **Landkarte** ein Rohstoffgebiet, um seine drei Vorkommen zu vergleichen. Mit einem Sammellager kannst du dort Gläubige und das Nilpferd beauftragen. Sie wechseln zu einer anderen erreichbaren Quelle, wenn die erste leer ist. **Zur Quelle** richtet die Kamera für den Abbau von Hand aus. Mit einer Ladung in der Pfote markiert das Spiel die passende Ablage. [Sammelaufträge und Bildvorlage](docs/Sammelauftraege.md)

Die fünf Kampagnen haben je drei Inselkarten mit eigenen Küsten, Höhenzügen, Übergängen und Startplätzen. Im Weidenland ziehen sich Waldarme um die Mühlen. Der Fjord hat lang gestreckte Landzungen, die Terrassen große Bewässerungsbecken, das Seeufer Sandbuchten und die Ägäis vorgelagerte Felsinseln. Neue Spielstände und erstmals betretene Kapitel nutzen diese Landschaften. Beim Fortsetzen lädt jedes schon besuchte Kapitel die Karte seines Spielstands.

Das Nilpferd wächst durch Lieferungen, Ernten und erfolgreiche Wachen. Wähle es mit der Pfote, um es zu streicheln. Lobe es nach einer Lieferung von Holz, Stein oder Nahrung. Jeden Rohstoff lernt es einzeln. Sobald es frei ist und ein erlernter Vorrat auf acht Einheiten oder weniger fällt, holt es selbst Nachschub. Es versorgt zuerst den knappsten erreichbaren Rohstoff. **Pause** unterbricht die Hilfe. **Sammeln** oder ein neues Lob setzt sie fort. Unter **Fortschritt** siehst du seine nächste Wachstumsstufe. Hilfe und Bündnisse bringen der Stadt mehr Glauben und Stimmung. Einschüchterung stärkt den Schutz durch Wachtürme und kostet Stimmung. Stadt und Kreatur zeigen ihre Ausrichtung auch im Spielbild.

Auf den neuen Inseln bringen Rivale und Kreatur eine eigene Bedrohung mit. Ivens Mooshirsch beansprucht Holz, Sigruns Runenwolf bringt Frost, Lins Wolkendrache verzögert die Ernte, Nurus Schilfschlange vertreibt Sammler und Nereas Windgreif zerstreut Wunderkraft. Die Aufträge zeigen den nächsten Angriff und mögliche Gegenmaßnahmen. Du kannst die Nachbarn mit einem Bündnis gewinnen oder ihre Wachen und Kreatur mit Wundern zurückdrängen.

Nach dem Bau des Grenzschreins kannst du passende Bündel **direkt zur Brücke** tragen. Später nimmt das Dorf dort auch seine Gabe entgegen. Teillieferungen bleiben beim Speichern erhalten. Am Übergang kannst du die offene Menge aus dem Lager bezahlen. Unter **Aufträge** führen **Bündnis vorbereiten** und **Wachen anvisieren** zu den beiden Wegen durch den Konflikt.

Bei einem Nachbardorf führt die Zielführung zum Bauplatz am örtlichen Weg, zum Ausbau oder zur fehlenden Gabe. Am Ende siehst du beide Dörfer und ihre Entscheidung. Zur ersten Landung auf der nächsten Insel kommen je verbündetem Dorf vier Nahrung und je Dorf mit vertriebenen Wachen vier Stein mit. Du kannst auch bleiben und weiterbauen.

![Die Abreise aus dem Weidental nach Bündnissen mit Weidenhain und Sonnenried. Acht Nahrung gehen mit zum Eichenpass.](docs/abreise-weidental.png)

Jedes Kapitel hat eigene Bitten und einen Nebenauftrag. Im Weidenland versorgst du Mühlen und Höfe, im Fjord sicherst du Winterquartiere und auf den Terrassen bringst du mehrere Reisernten ein. Am See verbinden neue Wege die Brunnen und Märkte. In der Ägäis produzierst du Öl und richtest ein Theater für das Fest der Winde ein. Das Auftragsfenster zeigt den Fortschritt jedes Ziels. Bei Straßenaufträgen zeigt die Führung einen angeschlossenen Weg zur Bestätigung. Für regionale Baupläne öffnet sie die erforderliche Forschung. Wähle Sägewerk, Steinhof, Schmiede oder Olivenpresse direkt an, um die nächste Schicht vorzuziehen. Das Objektmenü zeigt Einsatz und Ertrag, der Betrieb arbeitet danach automatisch weiter. Schon besuchte Kapitel aus älteren Spielständen behalten ihre Aufträge.

Tippe oben rechts auf **Zum Ziel** oder drücke **L3**, um Ort und Werkzeug für den aktuellen Auftrag zu finden. Unter **Menü → Aufträge** stehen die Aufgaben, ihr Fortschritt und die Bitten aus dem Dorf. Ein `!` auf der Karte kennzeichnet einen verfügbaren Nebenauftrag, ein `?` eine abholbare Belohnung. Der goldene Knopf zeigt die Aktion am ausgewählten Ziel. **Nilpferd** öffnet die Befehle zum Sammeln, Herbeirufen und zur Pflege. Das **Zauberbuch** zeigt die Voraussetzungen für Wunder. Der Fokusrahmen zeigt im Menü, welche Aktion du per Controller oder Tastatur bestätigst.

| Steuerung | Kamera und Ziel | Aktionen |
| --- | --- | --- |
| Touch | Boden ziehen, mit zwei Fingern schwenken, mit der Geste oder **+** und **−** zoomen, Ziel antippen | Objekt für sein Kontextmenü antippen, Quelle zum Greifen halten, **Aktionen** für das Auswahlrad |
| Controller | Linker Stick bewegt den Zielpunkt, rechter Stick schwenkt, Trigger zoomen | **LB/L1** öffnet das Auswahlrad, **X/Quadrat** das Baurad, **Y/Dreieck** das Wunderrad und **RB/R1** mit der Hand das Nilpferdrad. Stick oder Steuerkreuz wählen, **A/Kreuz** bestätigt, **B/Kreis** geht zurück. Beim Platzieren versetzt das Steuerkreuz den Plan feldweise, **X/Quadrat** dreht und **RB/R1** sucht einen freien Platz. **L3** zeigt das Auftragsziel auch beim Bauen. **R3** öffnet das Objektmenü. **Start** öffnet das Spielmenü. **Select/Options** öffnet die Landkarte. |
| Rechner | Maus zum Antippen und Ziehen, Mausrad oder **+** und **−** zum Zoomen, WASD zum Schwenken | Leertaste fürs Auswahlrad, 1–6 für Werkzeuge, E für die Aktion oder zum Halten beim Greifen, R zum Drehen, G für das Auftragsziel, H für das Werkzeugziel, M für die Landkarte, Tab für Kapitel, Q für Aufträge, P für Fortschritt, B fürs Baubuch und T für Forschung, I für Vorräte und F fürs Hauptmenü. Pfeiltasten und Enter bedienen Menüs. F12 speichert ein Spielbild. |

Das Spiel blendet die passenden Tasten an den Werkzeugen und bei der Bauvorschau ein. Die Übersicht steht im Spielmenü unter **Steuerung**.

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

Ein Entwicklungsbuild liegt unter `build/android/`. Für die Standalone-APK setze `HIPPOGROVE_STANDALONE=1`, `HIPPOGROVE_KEYSTORE` und `HIPPOGROVE_KEY_PASSWORD_FILE`. Der Schlüsselalias ist standardmäßig `hippogrove` und lässt sich mit `HIPPOGROVE_KEY_ALIAS` setzen. Die signierte APK landet unter `build/android/standalone/`. Updates der Standalone-App verwenden die Paket-ID `game.hippogrove.standalone` und dieselbe Signatur.

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

Die Hafenlaterne, der Wachfarn und die Umgebungs- und Aktionstöne stammen aus Waldwinkel 1.5. Die sechs Originalstücke umfassen „Die kleine Bucht“ für Titel und Tutorial sowie je eine eigene Besetzung und Bearbeitung für die fünf Landschaften. Beim Inselwechsel blendet die Musik über. Zum Rendern nutzt `art/build_music.py` FluidSynth, NumPy, pyFluidSynth, FFmpeg und die Instrumente von GeneralUser GS 2.0.3. Musik, Umgebung und Effekte lassen sich im Spielmenü unter **Ton** einzeln schalten. [Waldwinkel 1.5](https://github.com/bthginfo/waldwinkel/tree/8a98a76a2921690b533015e3f57fadccc2c66b24) · [Instrumentenbank und Lizenz](https://github.com/mrbumpy409/GeneralUser-GS/blob/684543d5e5efaef08d02be50dcda8d552478fa60/documentation/LICENSE.txt)

## Geprüfter Stand

Alle 42 Rust-Tests für 1.4.0 waren erfolgreich. Mit virtuellem Gamepad wurden das Tutorial von einem leeren Spielstand und die Kampagne ab Joras erster Lieferung geprüft. Im Weidental entstanden ein Brunnen für Weidenhain und ein Markt für Sonnenried. Beide Bündnisse lieferten beim ersten Anlegen am Eichenpass acht Nahrung. In einem vorbereiteten Korallenpass-Spielstand wurden Werkbänke erforscht, Steinhof und Olivenpresse gebaut und beide Ölpressungen über das Objektmenü ausgeführt.

Im Android-15-Emulator wurde die signierte 1.4.0 über die installierte 1.3.0 aktualisiert. Über das Hauptmenü ließ sich die begonnene Übung auf der Insel mit dem erhaltenen Bündel fortsetzen. Ein Test auf einem physischen Galaxy Tab S9 FE+ mit angeschlossenem Controller steht aus.

Die Rust-Tests spielen den Friedensweg durch die 15 Themenkapitel sowie die fünf Kapitel der Chronik und den Kampfweg im Weidental. Sie prüfen auch die Forschungs- und Produktionsketten, Geburten, Patrouillen, Wunder, die Menüführung und die Übernahme älterer Spielstände. [Spielaufnahmen und Prüfungen für 1.0](docs/V1.md) · [Stadtentwicklung](docs/Stadtentwicklung.md)
