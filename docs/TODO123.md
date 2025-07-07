# Torwell84 - Pending Tasks

## High Priority

## Medium Priority


## Future Ideas
- Real-time dashboard for resource usage
- System notifications on security warnings
- Integration with hardware security modules
- Mobile app version using Capacitor

## Completed Features
- Multiple simultaneous circuits per domain
- Connection retries use exponential backoff with a maximum total time.
- Each failed attempt increments `AppState.retry_counter` and is logged.
- Unit tests cover TorManager metrics functions
- Refactor TorManager error handling for better diagnostics

## Limitations
- Circuit metrics (active circuit count and age) will be added when arti-client provides a circuit listing API.
