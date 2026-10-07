type Profile = {
	authEndpoint?: string;
	relMes: string[];
};

const AUTH_ENDPOINT_HEADER_REGEX = /<([^>]+)>\s*;\s*rel="authorization_endpoint"/;

export const discoverProfile = async (domain: string) => {
	const profile: Profile = {
		authEndpoint: undefined,
		relMes: []
	};

	const res = await fetch(domain);

	const headerLink = res.headers.get('Link');
	if (headerLink) profile.authEndpoint = AUTH_ENDPOINT_HEADER_REGEX.exec(headerLink)?.[1];

	if (profile.authEndpoint) return profile;

	await new HTMLRewriter()
		.on('link[rel=authorization_endpoint][href]', {
			element(link) {
				const rel = link.getAttribute('rel');
				const href = link.getAttribute('href');
				if (!href || rel != 'authorization_endpoint') return;

				profile.authEndpoint = href;
			}
		})
		.on('a[rel][href]', {
			element(a) {
				const rel = a.getAttribute('rel');
				const href = a.getAttribute('href');
				if (!href || !rel || !rel.split(' ').includes('me')) return;

				profile.relMes.push(href);
			}
		})
		.transform(res)
		.text();

	return profile;
};
