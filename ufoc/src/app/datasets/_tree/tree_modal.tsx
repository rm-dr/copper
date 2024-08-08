import { Modal } from "@mantine/core";
import { ReactNode } from "react";

export function TreeModal(params: {
	children: ReactNode;
	title: ReactNode;
	keepOpen?: boolean;
	close: () => void;
	opened: boolean;
}) {
	return (
		<Modal
			opened={params.opened}
			onClose={() => {
				if (params.keepOpen !== true) {
					params.close();
				}
			}}
			title={params.title}
			centered
			overlayProps={{
				backgroundOpacity: 0.5,
				blur: 1,
			}}
		>
			{params.children}
		</Modal>
	);
}
