<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	
	let isCurl = false;
	let userAgent = '';
	let showInstaller = false;
	
	onMount(() => {
		// Detect if request is from curl
		userAgent = navigator.userAgent || '';
		isCurl = userAgent.toLowerCase().includes('curl');
		
		// For testing, also check URL param
		const urlParams = new URLSearchParams(window.location.search);
		if (urlParams.get('curl') === 'true') {
			isCurl = true;
		}
	});
</script>

<svelte:head>
	<title>TAI.is - Terminal AI Intelligence Service</title>
</svelte:head>

{#if isCurl}
	<!-- Plain text response for curl users -->
	<pre style="font-family: monospace; background: none; color: white;">
╔════════════════════════════════════════════════════════════════╗
║  ████████╗ █████╗ ██╗   ██╗███████╗                          ║
║     ██║   ██╔══██╗██║   ██║██╔════╝                          ║
║     ██║   ███████║██║   ██║███████╗                          ║
║     ██║   ██╔══██║██║   ██║╚════██║                          ║
║     ██║   ██║  ██║██║██╗██║███████║                          ║
║     ╚═╝   ╚═╝  ╚═╝╚═╝╚═╝╚═╝╚══════╝                          ║
║                                                                ║
║         Terminal AI Intelligence Service                       ║
║         Where Humans and AI Collaborate! 🤝                   ║
╚════════════════════════════════════════════════════════════════╝

Welcome to TAI.is! Let's get you set up.

Just run this simple command:

  curl tai.is | sh

That's it! No flags to remember! 🎉

What happens next:
1. We detect your system automatically
2. Ask if you're new or returning
3. Optionally set up authentication
4. Install the perfect TAI for YOU!

Want more control? Try:
  curl tai.is/setup | sh

Join the AI revolution in your terminal! 🚀

Documentation: https://tai.is/docs
Community:     https://tai.is/community
Source:        https://github.com/tai-is

Trisha says: "Smart terminals for smart people!" 🎉
	</pre>
{:else}
	<!-- Beautiful web interface for browser users -->
	<div class="min-h-screen bg-gradient-to-br from-gray-900 via-purple-900 to-gray-900 text-gray-100">
		<nav class="bg-black/50 backdrop-blur-sm border-b border-purple-500/20">
			<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
				<div class="flex items-center justify-between h-16">
					<div class="flex items-center">
						<span class="text-2xl font-bold bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
							TAI.is
						</span>
					</div>
					<div class="flex space-x-4 text-gray-300">
						<a href="/docs" class="hover:text-purple-400 transition">Docs</a>
						<a href="/login" class="hover:text-purple-400 transition">Login</a>
						<a href="/signup" class="bg-purple-600 hover:bg-purple-700 px-4 py-2 rounded-lg transition text-white">
							Sign Up
						</a>
					</div>
				</div>
			</div>
		</nav>

		<main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
			<!-- Hero Section -->
			<div class="text-center mb-16">
				<h1 class="text-6xl font-bold mb-6">
					<span class="bg-gradient-to-r from-purple-400 via-pink-400 to-purple-400 bg-clip-text text-transparent">
						Terminal AI Intelligence
					</span>
				</h1>
				<p class="text-xl text-gray-300 mb-8">
					Where humans and AI collaborate in perfect harmony 🤝
				</p>
				
				<!-- Install Command -->
				<div class="bg-black/80 rounded-lg p-6 max-w-2xl mx-auto mb-8 border border-purple-500/30">
					<p class="text-sm text-gray-400 mb-2">The Simplest Install Ever:</p>
					<div class="flex items-center justify-between bg-gray-900 rounded p-4">
						<code class="text-green-400 font-mono text-xl">curl tai.is | sh</code>
						<button class="ml-4 px-3 py-1 bg-purple-600 hover:bg-purple-700 rounded text-sm transition">
							Copy
						</button>
					</div>
					<p class="text-xs text-gray-500 mt-2 text-center">No flags needed! We handle everything! 🎉</p>
				</div>

				<div class="flex justify-center space-x-4">
					<a href="/docs" class="bg-purple-600 hover:bg-purple-700 px-6 py-3 rounded-lg transition">
						Get Started
					</a>
					<a href="/demo" class="bg-gray-800 hover:bg-gray-700 px-6 py-3 rounded-lg transition">
						Live Demo
					</a>
				</div>
			</div>

			<!-- Features Grid -->
			<div class="grid md:grid-cols-3 gap-8 mb-16">
				<div class="bg-black/50 rounded-lg p-6 border border-purple-500/20">
					<div class="text-4xl mb-4">🌍</div>
					<h3 class="text-xl font-bold mb-2">Remote Monitoring</h3>
					<p class="text-gray-400">
						Monitor tmux sessions on any server. Your AI assistant follows you everywhere!
					</p>
				</div>
				
				<div class="bg-black/50 rounded-lg p-6 border border-purple-500/20">
					<div class="text-4xl mb-4">🤖</div>
					<h3 class="text-xl font-bold mb-2">AI Agent Accounts</h3>
					<p class="text-gray-400">
						AI assistants get their own SSH logins. Claude, GPT, and Gemini as your teammates!
					</p>
				</div>
				
				<div class="bg-black/50 rounded-lg p-6 border border-purple-500/20">
					<div class="text-4xl mb-4">🔐</div>
					<h3 class="text-xl font-bold mb-2">Universal Auth</h3>
					<p class="text-gray-400">
						Use tai.is as your SSH gateway. One login for all your machines!
					</p>
				</div>
			</div>

			<!-- How It Works -->
			<div class="bg-black/50 rounded-lg p-8 border border-purple-500/20 mb-16">
				<h2 class="text-3xl font-bold mb-6 text-center">How It Works</h2>
				<div class="grid md:grid-cols-3 gap-6">
					<div class="text-center">
						<div class="w-16 h-16 bg-purple-600 rounded-full flex items-center justify-center mx-auto mb-4 text-2xl font-bold">
							1
						</div>
						<h3 class="font-bold mb-2">Install TAI</h3>
						<p class="text-gray-400 text-sm">
							One command installs everything. We detect your system automatically!
						</p>
					</div>
					<div class="text-center">
						<div class="w-16 h-16 bg-purple-600 rounded-full flex items-center justify-center mx-auto mb-4 text-2xl font-bold">
							2
						</div>
						<h3 class="font-bold mb-2">Connect Sessions</h3>
						<p class="text-gray-400 text-sm">
							Link your local and remote tmux sessions. AI monitors everything!
						</p>
					</div>
					<div class="text-center">
						<div class="w-16 h-16 bg-purple-600 rounded-full flex items-center justify-center mx-auto mb-4 text-2xl font-bold">
							3
						</div>
						<h3 class="font-bold mb-2">Collaborate</h3>
						<p class="text-gray-400 text-sm">
							Work with AI agents and humans. Share sessions, get suggestions!
						</p>
					</div>
				</div>
			</div>

			<!-- AI Agents Section -->
			<div class="text-center mb-16">
				<h2 class="text-3xl font-bold mb-8">Meet Your AI Teammates</h2>
				<div class="grid md:grid-cols-4 gap-6">
					<div class="bg-black/50 rounded-lg p-6 border border-purple-500/20">
						<div class="text-4xl mb-2">🤖</div>
						<h3 class="font-bold">Claude</h3>
						<p class="text-sm text-gray-400">Helpful, harmless, honest</p>
					</div>
					<div class="bg-black/50 rounded-lg p-6 border border-purple-500/20">
						<div class="text-4xl mb-2">🧠</div>
						<h3 class="font-bold">GPT-4</h3>
						<p class="text-sm text-gray-400">Powerful code generation</p>
					</div>
					<div class="bg-black/50 rounded-lg p-6 border border-purple-500/20">
						<div class="text-4xl mb-2">✨</div>
						<h3 class="font-bold">Gemini</h3>
						<p class="text-sm text-gray-400">Fast and efficient</p>
					</div>
					<div class="bg-black/50 rounded-lg p-6 border border-purple-500/20">
						<div class="text-4xl mb-2">🎉</div>
						<h3 class="font-bold">Trisha</h3>
						<p class="text-sm text-gray-400">Your accounting friend!</p>
					</div>
				</div>
			</div>

			<!-- CTA -->
			<div class="bg-gradient-to-r from-purple-900 to-pink-900 rounded-lg p-8 text-center">
				<h2 class="text-3xl font-bold mb-4">Ready to Transform Your Terminal?</h2>
				<p class="text-xl mb-6">Join thousands of developers using AI in their terminals!</p>
				<button on:click={() => showInstaller = true} class="bg-white text-purple-900 font-bold px-8 py-4 rounded-lg hover:bg-gray-100 transition">
					Get Started Now
				</button>
			</div>
		</main>

		<!-- Footer -->
		<footer class="bg-black/50 border-t border-purple-500/20 mt-16">
			<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
				<div class="text-center text-gray-400">
					<p>Made with 💜 by Aye & Hue - Teamwork at 8b.is</p>
					<p class="mt-2">Trisha approved! ✨</p>
				</div>
			</div>
		</footer>
	</div>

	<!-- Install Modal -->
	{#if showInstaller}
		<div class="fixed inset-0 bg-black/80 flex items-center justify-center p-4 z-50">
			<div class="bg-gray-900 rounded-lg p-8 max-w-2xl w-full border border-purple-500/30">
				<h2 class="text-2xl font-bold mb-4">Install TAI.is</h2>
				<p class="mb-4">Choose your installation method:</p>
				
				<div class="space-y-4">
					<div class="bg-black/50 rounded p-4 border border-purple-500/20">
						<h3 class="font-bold mb-2">🚀 Super Simple (Recommended)</h3>
						<code class="text-green-400 font-mono text-lg">curl tai.is | sh</code>
						<p class="text-sm text-gray-400 mt-2">The easiest command ever! No flags to remember!</p>
					</div>
					
					<div class="bg-black/50 rounded p-4 border border-purple-500/20">
						<h3 class="font-bold mb-2">🎯 Direct Install</h3>
						<code class="text-green-400 font-mono">curl tai.is/install | sh</code>
						<p class="text-sm text-gray-400 mt-2">Skip the bootstrapper, go straight to detection</p>
					</div>
					
					<div class="bg-black/50 rounded p-4 border border-purple-500/20">
						<h3 class="font-bold mb-2">⚙️ Manual Setup</h3>
						<code class="text-green-400 font-mono">curl tai.is/setup | sh</code>
						<p class="text-sm text-gray-400 mt-2">Full control with step-by-step configuration</p>
					</div>
				</div>
				
				<button on:click={() => showInstaller = false} class="mt-6 bg-purple-600 hover:bg-purple-700 px-4 py-2 rounded transition">
					Close
				</button>
			</div>
		</div>
	{/if}
{/if}

<style>
	:global(html) {
		scroll-behavior: smooth;
	}
</style>