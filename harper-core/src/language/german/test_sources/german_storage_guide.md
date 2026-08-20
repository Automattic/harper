# Speicher- und Sicherungskonzept

## Überblick

Die Anwendung hält häufig genutzte Daten zuerst im Arbeitsspeicher und verschiebt ältere Einträge später in den Festplattenspeicher.
Der Hintergrundprozess überwacht Speicherplatz, Zwischenspeicher und Netzwerkverbindung, damit die Synchronisierung stabil bleibt.

## Betrieb

Bei hoher Last leert der Dienst zuerst den Zwischenspeicher und schreibt große Anhänge in den Massenspeicher.
Die Benutzeroberfläche zeigt klar an, wann der Arbeitsspeicher knapp wird und wann zusätzlicher Festplattenspeicher benötigt wird.

## Wartung

Vor jedem Update erstellt das System eine Sicherungskopie der Konfigurationsdatei.
Nach dem Neustart prüft das Team die Dateiberechtigungen und bestätigt die erfolgreiche Fehlerbehebung.
