import { text } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ request }) => {
	const userAgent = request.headers.get('user-agent') || '';
	const isCurl = userAgent.toLowerCase().includes('curl');
	const isWget = userAgent.toLowerCase().includes('wget');
	
	if (!isCurl && !isWget) {
		// Redirect browsers to the main page
		return new Response(null, {
			status: 302,
			headers: {
				'Location': '/'
			}
		});
	}
	
	// Return the existing setup script for direct access
	// This is the fallback when users go directly to /setup
	const setupScript = await fetch('https://raw.githubusercontent.com/8bit-wraith/tmux-ai-assistant/main/scripts/tai-setup.sh');
	const scriptContent = await setupScript.text();
	
	return text(scriptContent, {
		headers: {
			'Content-Type': 'text/plain; charset=utf-8',
			'X-TAI-Installer': 'direct-v1'
		}
	});
};