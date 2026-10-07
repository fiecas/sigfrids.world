import { discoverProfile } from '$lib/auth/discovery';
import { fail, type Actions } from '@sveltejs/kit';

export const actions: Actions = {
	default: async ({ request }) => {
		const data = await request.formData();
		const domain = data.get('domain');

		if (!domain) return fail(400);
		const url = domain.toString().replace(/^(?!https?:\/\/)(.*)/, 'https://$1');

		const d = discoverProfile(url);
	}
};
