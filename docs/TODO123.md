# Torwell84 - Pending Tasks

## High Priority
1. GeoIP integration for country detection is already implemented in TorManager

## Medium Priority
1. Refactor TorManager error handling for better diagnostics
2. Add unit tests for critical backend functions

## Future Ideas
- Integration with hardware security modules
- Mobile app version using Capacitor

## Completed Features
- Multiple simultaneous circuits per domain
- Connection retries use exponential backoff with a maximum total time.
- Each failed attempt increments `AppState.retry_counter` and is logged.

## Limitations
- Circuit metrics (active circuit count and age) will be added when arti-client provides a circuit listing API.
