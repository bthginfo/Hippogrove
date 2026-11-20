# Hippogrove · Entwicklungsplan

## Die Rolle als Nilpferdgott

Das letzte Gebet eines kleinen Flussdorfs weckt einen jungen Nilpferdgott. Anfangs zeigt er sich als kleine Wolke mit einer Nilpferdpfote. Mit ihr hebt er Holz auf, platziert neue Gebäude und wirkt Wunder. Auf der Erde begleitet ihn ein rundes, auf zwei Beinen laufendes Nilpferd, das lernt, welche Arbeit seiner Siedlung hilft. In den folgenden Kapiteln führen alte Schreine zu weiteren Gemeinden, deren Flüsse und Vorräte von rivalisierenden Mächten beansprucht werden.

## Der Spielablauf

1. **Holz, Stein und Nahrung sammeln.** Mit der Wolkenpfote hebst du Bündel und bringst sie zum Sammelplatz oder Lager. Gläubige übernehmen zugewiesene Sammel- und Feldarbeit. Das Nilpferd kann Material holen und schwere Bündel tragen. Alle drei Wege füllen denselben Vorrat.
2. **Das Dorf ausbauen.** Hütten schaffen Wohnraum, Felder liefern Nahrung, Lager fassen Vorräte und Schreine weiten den Einfluss aus. Vor dem Bau siehst du den Grundriss, die Kosten und freie Wege. Beim Bestätigen prüfen wir Einfluss, Boden, Hindernisse und Vorrat gemeinsam.
3. **Glauben gewinnen.** Versorgung und erfüllte Bitten stärken den Glauben. Erreichte Schwellen öffnen Plätze für weitere Schreine am Rand deiner Einflusszone. Von dort erreichst du neue Rohstoffquellen und Siedlungen. Wunder verbrauchen einen eigenen Vorrat an Wunderkraft, der sich durch aktive Schreine wieder füllt.
4. **Wunder wirken und Rivalen begegnen.** Ein Regenschauer bewässert Felder. Ein Windstoß vertreibt gegnerische Wachen. Die Zielvorschau zeigt Wirkung, Reichweite und Kosten. Ein rivalisierendes Dorf kann sich dir durch Hilfe und erfüllte Versprechen anschließen oder sich nach einer Niederlage ergeben. Die Entscheidung prägt seine Reaktion und kann später weitere Aufträge beeinflussen.

## Der erste spielbare Abschnitt im Weidental

Eine kleine Karte mit Fluss, Brücke, Startdorf und rivalisierendem Weiler trägt etwa 15 bis 20 Minuten Spiel. Der Startschrein gibt dir Raum für die ersten Bauplätze. Drei Aufträge führen durch den gesamten Ablauf.

| Auftrag | Ziel |
| --- | --- |
| Ein Zuhause am Wasser | Sammle die ersten Bündel, baue Hütte und Lager und gib dem Nilpferd seinen ersten Transportauftrag. |
| Die trockenen Felder | Lege ein Feld an, nutze den Regenschauer und ernte Nahrung. Das dankbare Dorf stärkt deinen Glauben und öffnet einen Schreinplatz an der Brücke. |
| Die Brücke zum Nachbarn | Liefere dem Weiler Nahrung und stelle den Übergang wieder her, um ihn zu überzeugen. Alternativ vertreibst du die Wachen mit dem Windstoß. Beide Wege schließen das Kapitel ab und erweitern das Einflussgebiet. |

| Fortschritt | Erste Freischaltung | Im weiteren Spiel |
| --- | --- | --- |
| Gott | Ein Talent vergrößert die Fläche des Regenschauers. | Weitere Hilfe- und Angriffswunder sowie Verbesserungen ihrer Fläche, Kosten und Wirkung. |
| Siedlung | Eine neue Dorfstufe schaltet den Werkstattbau frei. | Lager und Gebäude verbessern, neue Gebäudearten für Versorgung, Schutz und Glauben freischalten. |
| Kreatur | Durch einen ausgeführten Transportauftrag und Belohnung lernt das Nilpferd, größere Bündel zu tragen. | Wiederholte Aufgaben und Belohnungen entwickeln Sammeln, Tragen und Beschützen. |

Gott und Kreatur nehmen ihre Freischaltungen ins nächste Kapitel mit. Der Ausbau jedes Dorfs bleibt im jeweiligen Level gespeichert.

## Steuerung auf Handy und Tablet

| Handlung | Touch | Controller |
| --- | --- | --- |
| Kamera | Über freien Boden ziehen und mit zwei Fingern zoomen | Rechter Stick und Trigger |
| Ziel wählen | Antippen oder im aktiven Werkzeugmodus das Ziel ziehen | Linker Stick bewegt die Wolkenpfote |
| Bauen, zaubern, Kreatur beauftragen | Werkzeugleiste sowie Schaltflächen zum Bestätigen und Abbrechen | Steuerkreuz für Werkzeuge und Menüs, Südtaste zum Bestätigen, Osttaste zum Abbrechen |

Der Bodenring zeigt den tatsächlichen Trefferpunkt. Die Wolke sitzt versetzt, damit das Ziel unter dem Finger sichtbar bleibt. Kameragesten brechen eine begonnene Zielauswahl ab. Beim Verbinden oder Trennen des Controllers bleibt das gewählte Werkzeug verfügbar. Touch und Controller lösen dieselben Spielbefehle aus, einschließlich Pause, Auftragsanzeige und Fortsetzen. Das Layout passt sich breiten Handys und 4:3-Tablets an.

## Was wir aus Waldwinkel übernehmen

- Die orthografische Schrägansicht, warmes Holz, olivgrüne Flächen, sichtbare Modellfacetten und cremefarbene Touch-Flächen bilden die visuelle Grundlage. Die Proportionen und getrennten Gelenkpunkte des Tanukis dienen als Ausgangspunkt für das neue Nilpferdmodell. [Hofansicht](https://github.com/bthginfo/waldwinkel/blob/bd29e952dd3e511d2c9c37556ff1d399cad25d45/docs/hof.png) · [Tanuki in der Holzhütte](https://github.com/bthginfo/waldwinkel/blob/bd29e952dd3e511d2c9c37556ff1d399cad25d45/docs/holzhuette.png)
- Eingaben werden wie in Waldwinkel vor der Spiellogik in gemeinsame Befehle übersetzt. Für Platzierung und Auswahl dient der dortige Ansatz mit Weltstrahl und begehbarem Gelände als Vorlage. Das Raster berücksichtigt zusätzlich neue Gebäude und mehrere sammelnde Figuren. [Eingaben](https://github.com/bthginfo/waldwinkel/blob/bd29e952dd3e511d2c9c37556ff1d399cad25d45/src/input.rs) · [Weltauswahl](https://github.com/bthginfo/waldwinkel/blob/bd29e952dd3e511d2c9c37556ff1d399cad25d45/src/adventure.rs)
- Wie in Waldwinkel kommen Blender-Quellen nach `art/`, GLB-Dateien nach `assets/`, Spielbilder nach `docs/` und die Android-Buildwerkzeuge nach `tools/` und `packaging/`. Bevy-Systeme bilden Siedlung, Einfluss, Aufträge und Kreatur ab. Level und Gebäudetypen werden über Datendateien definiert. Der versionierte lokale Spielstand wird bei Bau, Questabschluss, Levelwechsel und beim Wechsel in den Hintergrund vollständig geschrieben und dann atomar ersetzt. [Blender- und Android-Ablauf in Waldwinkel](https://github.com/bthginfo/waldwinkel/blob/bd29e952dd3e511d2c9c37556ff1d399cad25d45/README.md)

## Reihenfolge der Umsetzung

1. **Geräteprototyp.** Eine kleine 3D-Szene mit Kamera, Wolkenpfote und Auswahlring auf Android sowie iOS und iPadOS starten. Waldwinkels Android-Paketierung dient als Ausgangspunkt. Auf beiden Plattformen Touch, Tasten, Sticks und das Anschließen und Trennen eines Controllers am laufenden Spiel prüfen. Für die nativen Controller-Eingaben sind Android-Eingabeereignisse und Apples Game Controller vorgesehen. [Android](https://developer.android.com/games/sdk/game-controller/controller-input) · [Apple](https://developer.apple.com/documentation/GameController/GCController) · [Bevy-Mobilbeispiele](https://github.com/bevyengine/bevy/tree/main/examples/mobile)
2. **Sammeln und Bauen.** Das Weidental gestalten und Licht, Größen und Kamerabild an der Referenz ausrichten. Die ersten Gebäude platzieren, Vorräte verrechnen und die Sammelaufträge für Gott, Gläubige und Kreatur anbinden. Das Nilpferd läuft, hebt ein Bündel auf und setzt es im Lager ab.
3. **Weidental abschließen.** Schreine, die beiden Wunder, die drei Aufträge und beide Wege zum Kapitelsieg verbinden. Je eine Freischaltung für Gott, Stadt und Kreatur sowie lokales Speichern und Fortsetzen einbauen.
4. **Kampagne ausbauen.** Auf dem mitteleuropäisch inspirierten Weidental folgen ein nordischer Fjord mit Wintervorräten und einem umkämpften Anleger, ostasiatisch inspirierte Reisterrassen mit Streit um die Wasserverteilung und ein Kapitel an einem ostafrikanisch inspirierten See mit Regen- und Trockenzeit. Jede Region bekommt eine eigene Architektur, Palette, Legende, Questkette und einen Rivalen. Für jede Region entsteht vor der Modellierung ein eigenes Referenzbild.

## Woran wir den ersten spielbaren Abschnitt prüfen

- **Bildvergleich.** Das Referenzbild gibt Formen, Farben, Maßstab und Kameraperspektive vor. In Bevy bauen wir dieselbe Spielsituation nach und vergleichen Aufnahmen aus der normalen Spielkamera. Auf einem Handybild und einem Tabletbild müssen Nilpferd, Wolkenpfote, Bodenring, Schrein und Bauvorschau auf Anhieb auseinanderzuhalten sein. Die Einflussgrenze im Spiel umfasst alle freigegebenen Bauplätze. Animationen zeigen klar den Fußkontakt und das Greifen und Absetzen eines Bündels. [Weidental-Referenz](docs/weidental-referenz.png)
- **Spielregeln.** Jeder Sammelweg liefert genau einmal ins Lager. Eine gültige Platzierung verbraucht genau einmal Material, unzulässige Plätze bleiben unverändert. Der neue Schrein macht zusätzliche Plätze verfügbar. Die Vorschau kennzeichnet blockierte Bauplätze zusätzlich zur Farbe mit einem Symbol.
- **Durchspielen auf Geräten.** Den Abschnitt auf Android- und Apple-Hardware in Handy- und Tabletgröße spielen. Beide Eingabearten müssen den vollständigen Ablauf bis zu einem Kapitelsieg tragen. Dazu gehören Dialoge, Bauen, Zielvorschau, Abbruch, Wunder und Kreaturaufträge. Beide Enden prüfen, den Controller während einer Bauvorschau wechseln und den Spielstand nach Hintergrundwechsel und Neustart laden. Auf dem schwächsten Testgerät streben wir beim Spielen mindestens 30 Bilder pro Sekunde an.
