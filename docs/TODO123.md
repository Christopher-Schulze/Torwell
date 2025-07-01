# Torwell84 - Pending Tasks

## High Priority
1. Implement GeoIP integration for country detection in Tor relays
2. Add real traffic metrics to StatusCard component
3. Create persistent log storage (file-based)

## Medium Priority
1. Refactor TorManager error handling for better diagnostics
2. Add unit tests for critical backend functions
3. // Darkmode entfernt

## Future Ideas
- Integration with hardware security modules
- Mobile app version using Capacitor

## Completed Features
- Multiple simultaneous circuits per domain
- Connection retries use exponential backoff with a maximum total time.
- Each failed attempt increments `AppState.retry_counter` and is logged.
