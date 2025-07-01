<script lang="ts">
        import { X, Download, Copy, Trash2, FolderOpen } from 'lucide-svelte';
        import { createEventDispatcher, onMount, tick } from 'svelte';
        import { invoke } from '@tauri-apps/api';
        import { open } from '@tauri-apps/api/shell';

	export let show = false;

	const dispatch = createEventDispatcher();

	interface LogEntry {
		type: string;
		timestamp: string;
		message: string;
		level: string;
	}

        let activeTab = 'all';
        let levelFilter = 'all';
        let logs: LogEntry[] = [];
        let isLoading = false;
       let isClearing = false;
       let logFilePath = '';
       let closeButton: HTMLButtonElement | null = null;

        $: filteredByType = activeTab === 'all' ? logs : logs.filter(log => log.type === activeTab);
        $: filteredLogs = levelFilter === 'all' ? filteredByType : filteredByType.filter(log => log.level === levelFilter);

        $: if (show) {
                loadLogs();
                fetchLogFilePath();
                tick().then(() => closeButton && closeButton.focus());
        }

        async function loadLogs() {
                isLoading = true;
                try {
                        const response: any[] = await invoke('get_logs');
			logs = response.map(entry => ({
				type: entry.level.toLowerCase() === 'info' ? 'connection' : 'system',
				timestamp: new Date(entry.timestamp).toLocaleString(),
				message: entry.message,
				level: entry.level
			}));
		} catch (error) {
			console.error('Failed to load logs:', error);
			logs = [
				{ type: 'system', timestamp: new Date().toLocaleString(), message: 'Failed to load logs from backend', level: 'ERROR' }
			];
		} finally {
			isLoading = false;
                }
        }

        async function fetchLogFilePath() {
                try {
                        logFilePath = await invoke('get_log_file_path');
                } catch (error) {
                        console.error('Failed to get log file path', error);
                }
        }

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			dispatch('close');
		}
	}

	function downloadLogs() {
		const logText = filteredLogs.map(log => `[${log.timestamp}] [${log.level || 'INFO'}] ${log.message}`).join('\n');
		const blob = new Blob([logText], { type: 'text/plain' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5);
		const filename = activeTab === 'all' ? `torwell-all-logs-${timestamp}.txt` : `torwell-${activeTab}-logs-${timestamp}.txt`;
		a.download = filename;
		a.click();
		URL.revokeObjectURL(url);
	}

	function copyLogs() {
		const logText = filteredLogs.map(log => `[${log.timestamp}] [${log.level || 'INFO'}] ${log.message}`).join('\n');
		navigator.clipboard.writeText(logText);
	}

        async function clearLogs() {
                if (isClearing) return;
                isClearing = true;
                try {
                        await invoke('clear_logs');
                        console.log('Logs cleared successfully');
                        logs = [];
                } catch (error) {
                        console.error('Failed to clear logs:', error);
                } finally {
                        isClearing = false;
                }
        }

        function openLogFile() {
                if (logFilePath) {
                        open(logFilePath);
                }
        }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if show}
        <div class="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
                <div
                        class="bg-black/80 backdrop-blur-3xl rounded-2xl border border-white/10 w-full max-w-4xl max-h-[80vh] overflow-hidden"
                        role="dialog"
                        aria-modal="true"
                        aria-labelledby="logs-modal-title"
                >
			<!-- Header -->
			<div class="flex items-center justify-between p-6 border-b border-white/10">
                                <h2 id="logs-modal-title" class="text-xl font-semibold text-white">Logs</h2>
                                <button
                                        class="text-gray-100 hover:text-white transition-colors"
                                        on:click={() => dispatch('close')}
                                        aria-label="Close logs"
                                        bind:this={closeButton}
                                >
					<X size={24} />
				</button>
			</div>

			<!-- Tabs -->
                        <div class="flex border-b border-white/10">
                                {#each [{ id: 'all', label: 'All Logs' }, { id: 'connection', label: 'Connection Logs' }, { id: 'system', label: 'System Logs' }] as tab}
                                        <button
                                                class="px-6 py-3 text-sm font-medium transition-colors {activeTab === tab.id
                                                        ? 'text-blue-400 border-b-2 border-blue-400'
                                                        : 'text-gray-100 hover:text-white'}"
                                                on:click={() => (activeTab = tab.id)}
                                                aria-label={tab.label}
                                        >
                                                {tab.label}
                                        </button>
                                {/each}
                        </div>

                        <div class="flex items-center gap-2 p-2 border-b border-white/10">
                                <label class="text-sm text-gray-100" for="logs-level-filter">Level:</label>
                                <select id="logs-level-filter" bind:value={levelFilter} class="bg-gray-800 text-white text-sm rounded p-1" aria-label="Filter logs by level">
                                        <option value="all">All</option>
                                        <option value="INFO">Info</option>
                                        <option value="WARN">Warn</option>
                                        <option value="ERROR">Error</option>
                                </select>
                        </div>

			<!-- Log Content -->
			<div class="p-6 max-h-96 overflow-y-auto">
                                {#if isLoading}
                                        <div class="text-center text-gray-100 py-8">
						<div class="animate-spin w-6 h-6 border-2 border-blue-400 border-t-transparent rounded-full mx-auto mb-2"></div>
						Loading logs...
					</div>
                                {:else if filteredLogs.length === 0}
                                        <div class="text-center text-gray-100 py-8">
						No logs available
					</div>
				{:else}
					<div class="space-y-2">
                                                <div class="text-sm text-gray-100 font-mono">
                                                        {#each filteredLogs as log}
                                                                <div class="{log.level === 'ERROR' ? 'text-red-400' : log.level === 'WARN' ? 'text-yellow-400' : 'text-blue-400'}">
                                                                        [{log.timestamp}] [{log.level}] {log.message}
                                                                </div>
                                                        {/each}
						</div>
					</div>
				{/if}
			</div>

			<!-- Footer -->
			<div class="flex items-center justify-between p-6 border-t border-white/10">
                                <div class="flex gap-2">
                                        <button
                                                class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-sm flex items-center gap-2"
                                                on:click={downloadLogs}
                                                aria-label="Download logs"
                                        >
                                                <Download size={16} /> Download
                                        </button>
                                        <button
                                                class="px-4 py-2 bg-gray-700 text-white rounded-lg hover:bg-gray-800 transition-colors text-sm flex items-center gap-2"
                                                on:click={openLogFile}
                                                aria-label="Open log file"
                                        >
                                                <FolderOpen size={16} /> Open File
                                        </button>
                                        <button
                                                class="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors text-sm flex items-center gap-2"
                                                on:click={copyLogs}
                                                aria-label="Copy logs"
                                        >
                                                <Copy size={16} /> Copy
					</button>
				</div>
                                <button
                                        class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors text-sm flex items-center gap-2 {isClearing ? 'opacity-50 cursor-not-allowed' : ''}"
                                        on:click={clearLogs}
                                        disabled={isClearing}
                                        aria-label="Clear logs"
                                >
                                        <Trash2 size={16} /> {isClearing ? 'Clearing...' : 'Clear Logs'}
                                </button>
                        </div>
                        <p class="text-xs text-gray-100 px-6 pb-4">Log file: {logFilePath}</p>
                </div>
        </div>
{/if}
