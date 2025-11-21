# UI Redesign: "Cyber-Stealth" & Svelte Physics

## Ziel
Implementierung eines hochmodernen, performanten UI-Designs unter dem Codenamen "Cyber-Stealth". Fokus liegt auf "Super Ultra Sexy Design", maximaler Smoothness und Fehlerfreiheit.

## Design-Philosophie
- **Theme**: "Void" Black Backgrounds mit subtilen Noise-Texturen.
- **Akzente**: Neon Cyber-Green (#00ff41) & Electric Purple (#bc13fe) für Interaktionen.
- **Material**: High-End Glassmorphism (Backdrop-Blur, weiße Opazität 5-10%).
- **Typografie**: Mix aus Sans-Serif (Inter/Roboto) für Lesbarkeit und Monospace (Fira Code/JetBrains) für Daten.

## Technische Umsetzung (No Framer Motion)
Wir entfernen (oder vermeiden) schwere Animations-Bibliotheken wie Framer Motion und nutzen **Svelte Native Physics**:
- `svelte/motion`: `spring` und `tweened` für physisch korrekte Bewegungen (keine linearen Tweens, sondern Masse & Federn).
- `svelte/transition`: Performante CSS-basierte Transitions.

## Komponenten
1. **CyberButton**: Magnetischer Hover-Effekt, Glow-Border, "Clicky" Spring-Animation.
2. **GlassCard**: Container mit Blur, feinem Border-Gradient und Noise.
3. **PageTransition**: Nahtlose Übergänge zwischen Routen.

## Maßnahmen
1. Tailwind Config erweitern (Colors, Animations).
2. CSS Variablen definieren.
3. Core-Components bauen.
4. Showcase-Page erstellen.
