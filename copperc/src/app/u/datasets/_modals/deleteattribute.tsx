import { Button, Text, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useForm } from "@mantine/form";
import { ModalBaseSmall, modalStyle } from "@/components/modalbase";
import { useMutation } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";

export function useDeleteAttributeModal(params: {
	attribute_id: number;
	attribute_name: string;
	onSuccess: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);

	const form = useForm({
		mode: "uncontrolled",
		initialValues: {
			attribute_name: "",
		},
		validate: {
			attribute_name: (value) => {
				if (value.trim().length === 0) {
					return "This field is required";
				}

				if (value !== params.attribute_name) {
					return "Attribute name doesn't match";
				}

				return null;
			},
		},
	});

	const doDelete = useMutation({
		mutationFn: async () => {
			return await edgeclient.DELETE("/attribute/{attribute_id}", {
				params: { path: { attribute_id: params.attribute_id } },
			});
		},

		onSuccess: async (res) => {
			if (res.response.status === 200) {
				reset();
				params.onSuccess();
			} else {
				throw new Error(res.error);
			}
		},

		onError: (err) => {
			throw err;
		},
	});

	const reset = () => {
		doDelete.reset();
		form.reset();
		close();
	};

	return {
		open,
		modal: (
			<ModalBaseSmall
				opened={opened}
				close={reset}
				title="Delete attribute"
				keepOpen={doDelete.isPending}
			>
				<div
					style={{
						marginBottom: "1rem",
					}}
				>
					<Text c="red" size="sm">
						This action will irreversably destroy data.
					</Text>
					<Text c="red" size="sm">
						Enter
						<Text c="orange" span>{` ${params.attribute_name} `}</Text>
						below to confirm.
					</Text>
				</div>

				<form
					onSubmit={form.onSubmit(() => {
						doDelete.mutate();
					})}
				>
					<div className={modalStyle.modal_input_container}>
						<TextInput
							data-autofocus
							placeholder="Enter attribute name"
							disabled={doDelete.isPending}
							key={form.key("attribute_name")}
							{...form.getInputProps("attribute_name")}
						/>

						<Button.Group>
							<Button
								variant="light"
								fullWidth
								color="red"
								onClick={reset}
								disabled={doDelete.isPending}
							>
								Cancel
							</Button>
							<Button
								variant="filled"
								fullWidth
								color="red"
								loading={doDelete.isPending}
								type="submit"
							>
								Confirm
							</Button>
						</Button.Group>

						{doDelete.error ? (
							<Text c="red" ta="center">
								{doDelete.error.message}
							</Text>
						) : null}
					</div>
				</form>
			</ModalBaseSmall>
		),
	};
}
