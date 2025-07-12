import { text } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

// This handles "curl tai.is/sh" for people who add /sh out of habit!
export const GET: RequestHandler = async ({ request }) => {
	const userAgent = request.headers.get('user-agent') || '';
	const isCurl = userAgent.toLowerCase().includes('curl');
	const isWget = userAgent.toLowerCase().includes('wget');
	
	if (!isCurl && !isWget) {
		// Redirect browsers to home
		return new Response(null, {
			status: 302,
			headers: {
				'Location': '/'
			}
		});
	}
	
	// Return the same bootstrapper but with a fun message!
	const bootstrapper = `#!/bin/sh
# TAI.is - You added /sh but that's OK! We got you! 😄
# Next time just use: curl tai.is | sh

echo "🎉 Welcome to TAI.is!"
echo "💡 Pro tip: You can just use 'curl tai.is | sh' next time!"
echo ""

# Download and run the smart installer
curl -sSL tai.is/install | sh
`;

	return text(bootstrapper, {
		headers: {
			'Content-Type': 'text/plain; charset=utf-8',
			'X-TAI-Installer': 'sh-redirect-v1'
		}
	});
};