import { text } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ request }) => {
	const userAgent = request.headers.get('user-agent') || '';
	const isCurl = userAgent.toLowerCase().includes('curl');
	const isWget = userAgent.toLowerCase().includes('wget');
	
	if (!isCurl && !isWget) {
		// For browsers, redirect to the home page
		return new Response(null, {
			status: 302,
			headers: {
				'Location': '/'
			}
		});
	}
	
	// For curl/wget, return the super simple bootstrapper!
	const bootstrapper = `#!/bin/sh
# TAI.is Bootstrapper - The simplest installer ever! 🚀
# This tiny script does all the heavy lifting for you!

echo "🎉 Welcome to TAI.is!"
echo ""

# Download and run the smart installer with all the proper flags
curl -sSL tai.is/install | sh

# That's it! The installer will:
# 1. Detect your system
# 2. Ask a few questions
# 3. Install the perfect TAI for you!
`;

	return text(bootstrapper, {
		headers: {
			'Content-Type': 'text/plain; charset=utf-8',
			'X-TAI-Installer': 'bootstrapper-v1'
		}
	});
};