<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import * as THREE from 'three';
	import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
	import type { Panel3D } from '$lib/types';
	import {
		getExplodeOffset,
		panelColor,
		computeBoundingBox,
		boundingBoxCenter,
		boundingBoxMaxExtent
	} from './scene-utils';

	let {
		panels = [],
		exploded = false,
		wireframe = false,
		selectedPanel = null,
		onSelectPanel = (_label: string | null) => {}
	}: {
		panels: Panel3D[];
		exploded: boolean;
		wireframe: boolean;
		selectedPanel: string | null;
		onSelectPanel: (label: string | null) => void;
	} = $props();

	let container: HTMLDivElement;
	let renderer: THREE.WebGLRenderer;
	let scene: THREE.Scene;
	let camera: THREE.PerspectiveCamera;
	let controls: OrbitControls;
	let panelGroup: THREE.Group;
	let animFrameId: number;
	let resizeObserver: ResizeObserver;
	let raycaster: THREE.Raycaster;
	let pointer: THREE.Vector2;

	function buildMeshes() {
		// Clear existing meshes
		while (panelGroup.children.length > 0) {
			const child = panelGroup.children[0] as THREE.Mesh;
			child.geometry?.dispose();
			if (child.material) {
				if (Array.isArray(child.material)) {
					child.material.forEach((m) => m.dispose());
				} else {
					child.material.dispose();
				}
			}
			panelGroup.remove(child);
		}

		if (panels.length === 0) return;

		const bb = computeBoundingBox(panels);
		if (!bb) return;

		const center = boundingBoxCenter(bb);

		for (const panel of panels) {
			const geo = new THREE.BoxGeometry(panel.width, panel.height, panel.depth);
			const mat = new THREE.MeshStandardMaterial({
				color: panelColor(panel.color),
				wireframe: wireframe,
				transparent: wireframe,
				opacity: wireframe ? 0.3 : 1.0,
				roughness: 0.7,
				metalness: 0.0
			});

			if (selectedPanel === panel.label) {
				mat.emissive = new THREE.Color(0xe94560);
				mat.emissiveIntensity = 0.4;
			}

			const mesh = new THREE.Mesh(geo, mat);
			const offset = getExplodeOffset(panel.label, exploded);
			mesh.position.set(
				panel.x - center.x + offset.x,
				panel.y - center.y + offset.y,
				panel.z - center.z + offset.z
			);
			mesh.userData.label = panel.label;
			panelGroup.add(mesh);
		}

		// Position camera to fit
		const size = boundingBoxMaxExtent(bb);
		const dist = size * 1.8;
		camera.position.set(dist * 0.7, dist * 0.5, dist * 0.7);
		camera.lookAt(0, 0, 0);
		controls.target.set(0, 0, 0);
		controls.update();
	}

	function updateMaterials() {
		for (const child of panelGroup.children) {
			const mesh = child as THREE.Mesh;
			const mat = mesh.material as THREE.MeshStandardMaterial;
			mat.wireframe = wireframe;
			mat.transparent = wireframe;
			mat.opacity = wireframe ? 0.3 : 1.0;

			if (selectedPanel === mesh.userData.label) {
				mat.emissive = new THREE.Color(0xe94560);
				mat.emissiveIntensity = 0.4;
			} else {
				mat.emissive = new THREE.Color(0x000000);
				mat.emissiveIntensity = 0;
			}
		}
	}

	function updatePositions() {
		if (panels.length === 0) return;

		const bb = computeBoundingBox(panels);
		if (!bb) return;
		const center = boundingBoxCenter(bb);

		for (let i = 0; i < panelGroup.children.length; i++) {
			const mesh = panelGroup.children[i] as THREE.Mesh;
			const panel = panels[i];
			if (!panel) continue;
			const offset = getExplodeOffset(panel.label, exploded);
			mesh.position.set(
				panel.x - center.x + offset.x,
				panel.y - center.y + offset.y,
				panel.z - center.z + offset.z
			);
		}
	}

	function handleClick(event: MouseEvent) {
		if (!container || !renderer) return;

		const rect = container.getBoundingClientRect();
		pointer.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
		pointer.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

		raycaster.setFromCamera(pointer, camera);
		const intersects = raycaster.intersectObjects(panelGroup.children);

		if (intersects.length > 0) {
			const label = intersects[0].object.userData.label;
			onSelectPanel(selectedPanel === label ? null : label);
		} else {
			onSelectPanel(null);
		}
	}

	function animate() {
		animFrameId = requestAnimationFrame(animate);
		controls.update();
		renderer.render(scene, camera);
	}

	function handleResize() {
		if (!container || !renderer) return;
		const w = container.clientWidth;
		const h = container.clientHeight;
		camera.aspect = w / h;
		camera.updateProjectionMatrix();
		renderer.setSize(w, h);
	}

	onMount(() => {
		scene = new THREE.Scene();
		scene.background = new THREE.Color(0x1a1a2e);

		camera = new THREE.PerspectiveCamera(45, container.clientWidth / container.clientHeight, 0.1, 1000);

		renderer = new THREE.WebGLRenderer({ antialias: true });
		renderer.setPixelRatio(window.devicePixelRatio);
		renderer.setSize(container.clientWidth, container.clientHeight);
		container.appendChild(renderer.domElement);

		controls = new OrbitControls(camera, renderer.domElement);
		controls.enableDamping = true;
		controls.dampingFactor = 0.1;

		// Lighting
		const ambient = new THREE.AmbientLight(0xffffff, 0.6);
		scene.add(ambient);

		const dirLight = new THREE.DirectionalLight(0xffffff, 0.8);
		dirLight.position.set(10, 15, 10);
		scene.add(dirLight);

		// Grid helper for spatial reference
		const gridSize = 60;
		const grid = new THREE.GridHelper(gridSize, 30, 0x333355, 0x222244);
		grid.position.y = -0.01;
		scene.add(grid);

		panelGroup = new THREE.Group();
		scene.add(panelGroup);

		raycaster = new THREE.Raycaster();
		pointer = new THREE.Vector2();

		renderer.domElement.addEventListener('click', handleClick);

		resizeObserver = new ResizeObserver(handleResize);
		resizeObserver.observe(container);

		buildMeshes();
		animate();
	});

	onDestroy(() => {
		if (animFrameId) cancelAnimationFrame(animFrameId);
		renderer?.domElement?.removeEventListener('click', handleClick);
		resizeObserver?.disconnect();
		controls?.dispose();

		// Dispose all meshes
		if (panelGroup) {
			while (panelGroup.children.length > 0) {
				const child = panelGroup.children[0] as THREE.Mesh;
				child.geometry?.dispose();
				if (child.material) {
					if (Array.isArray(child.material)) {
						child.material.forEach((m) => m.dispose());
					} else {
						child.material.dispose();
					}
				}
				panelGroup.remove(child);
			}
		}

		renderer?.dispose();
	});

	// React to panels changing — rebuild geometry
	$effect(() => {
		// Access panels to register dependency
		const _p = panels;
		if (panelGroup) buildMeshes();
	});

	// React to exploded state — update positions
	$effect(() => {
		const _e = exploded;
		if (panelGroup && panels.length > 0) updatePositions();
	});

	// React to wireframe/selectedPanel — update materials only
	$effect(() => {
		const _w = wireframe;
		const _s = selectedPanel;
		if (panelGroup) updateMaterials();
	});
</script>

<div bind:this={container} class="w-full h-full min-h-[400px] rounded-lg overflow-hidden"></div>
