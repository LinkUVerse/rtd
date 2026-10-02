// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { toast } from 'react-hot-toast';

import { useCreateKioskMutation } from '../../mutations/kiosk';
import { Button } from '../Base/Button';

export function KioskCreation({ onCreate }: { onCreate: () => void }) {
	const createKiosk = useCreateKioskMutation({
		onSuccess: () => {
			onCreate();
			toast.success('Kiosk created successfully');
		},
	});

	return (
		<div className="min-h-[70vh] container py-24 gap-4 mt-6">
			<div className="lg:w-7/12 mx-auto">
				<h2 className="font-bold text-3xl mb-6">Create a Rtd Kiosk</h2>
				<p className="pb-3">
					<strong>There’s no kiosk for your address yet.</strong> Create a kiosk to store your
					digital assets and list them for sale on the Rtd network. Anyone can view your kiosk and
					the assets you place in it.
				</p>
				<p className="pb-3">
				Connect your wallet to an RTD network that runs this kiosk package and ensure the
				address has enough RTD for gas. Obtain test tokens from that network's operator.
				</p>
				<p className="pb-3">
					When you click <strong>Create Kiosk</strong>, your wallet opens. Click{' '}
					<strong>Approve</strong> to allow the app to create a kiosk for the connected wallet
					address.
				</p>
				<Button
					loading={createKiosk.isPending}
					onClick={() => createKiosk.mutate()}
					className="mt-3 px-12 bg-primary text-white"
				>
					Create Kiosk
				</Button>
			</div>
		</div>
	);
}
