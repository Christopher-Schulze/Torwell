# UI-Design

Torwell84 nutzt das Plugin `tailwindcss-glassmorphism`, um transparente Oberflächen mit weichen Unschärfe-Effekten zu gestalten. Die folgenden Klassen stehen zur Verfügung und können wie normale Tailwind-Utilities verwendet werden:

- `glass-none` – deaktiviert jeglichen Glaseffekt
- `glass-sm` – dezenter Glaseffekt
- `glass` – Standard-Glaseffekt
- `glass-md` – mittelstarker Glaseffekt
- `glass-lg` – stärkerer Glaseffekt
- `glass-xl` – ausgeprägter Glaseffekt
- `glass-2xl` – intensivster Glaseffekt

Die jeweilige Stufe bestimmt Unschärfe, Hintergrundfarbe und Transparenz. Durch Kombination mit weiteren Tailwind-Klassen lassen sich z.B. abgerundete Ecken oder Ränder definieren.

## Barrierefreiheit

Gemäß den Abschnitten zur Barrierefreiheit in `DOCUMENTATION.md` sollten alle interaktiven Elemente aussagekräftige `aria-label` Attribute besitzen und ausreichend Farbkontrast aufweisen. Bei modalen Dialogen wandert der Tastaturfokus auf den Schließ-Button, damit Tastaturnutzer das Fenster direkt schließen können.

Farbwahl und Kontrast müssen dabei den WCAG&nbsp;2.1 AA Richtlinien entsprechen, um eine gute Lesbarkeit zu gewährleisten. Insbesondere bei der Verwendung transparenter Hintergründe sollte darauf geachtet werden, dass Text nicht zu wenig Kontrast zum Hintergrund besitzt.
