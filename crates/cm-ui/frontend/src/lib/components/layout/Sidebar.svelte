<script lang="ts">
	import { page } from '$app/state';
	import { project, cabinetCount, totalPartCount } from '$lib/stores/project';
	import { sidebarCollapsed } from '$lib/stores/ui';

	const navItems = [
		{ href: '/', label: 'Home', icon: 'H' },
		{ href: '/project', label: 'Project', icon: 'P' },
		{ href: '/cabinets', label: 'Cabinets', icon: 'C' },
		{ href: '/materials', label: 'Materials', icon: 'M' },
		{ href: '/cutlist', label: 'Cut List', icon: 'L' },
		{ href: '/nesting', label: 'Nesting', icon: 'N' },
		{ href: '/preview', label: '3D Preview', icon: '3' },
		{ href: '/generate', label: 'Generate', icon: 'G' }
	];
</script>

<nav
	class="flex flex-col bg-bg-secondary border-r border-border h-full transition-all duration-200"
	class:w-48={!$sidebarCollapsed}
	class:w-12={$sidebarCollapsed}
>
	<div class="flex items-center justify-between p-3 border-b border-border">
		{#if !$sidebarCollapsed}
			<span class="font-bold text-sm text-text-primary">Cabinet Maker</span>
		{/if}
		<button
			class="text-text-secondary hover:text-text-primary text-xs px-1"
			onclick={() => sidebarCollapsed.update((v) => !v)}
		>
			{$sidebarCollapsed ? '>' : '<'}
		</button>
	</div>

	<div class="flex-1 py-2">
		{#each navItems as item}
			{@const isActive = page.url.pathname === item.href}
			<a
				href={item.href}
				class="flex items-center gap-2 px-3 py-2 text-sm transition-colors"
				class:bg-surface={isActive}
				class:text-accent={isActive}
				class:text-text-secondary={!isActive}
				class:hover:bg-surface-hover={!isActive}
				class:hover:text-text-primary={!isActive}
			>
				<span class="w-5 h-5 flex items-center justify-center text-xs font-mono font-bold rounded bg-border">
					{item.icon}
				</span>
				{#if !$sidebarCollapsed}
					<span>{item.label}</span>
				{/if}
			</a>
		{/each}
	</div>

	{#if !$sidebarCollapsed && $project}
		<div class="p-3 border-t border-border text-xs text-text-secondary space-y-1">
			<div>Cabinets: {$cabinetCount}</div>
			<div>Parts: {$totalPartCount}</div>
		</div>
	{/if}
</nav>
