import { createRouter, createWebHistory } from 'vue-router';

import { store } from '~/store';

import Dashboard from '~/components/dashboard/Dashboard.vue';
import Outages from '~/components/outages/Outages.vue';
import Outage from '~/components/outages/Outage.vue';
import History from '~/components/outages/History.vue';
import Checks from '~/components/checks/Checks.vue';
import Check from '~/components/checks/Check.vue';
import CheckForm from '~/components/checks/Form.vue';
import Groups from '~/components/groups/Groups.vue';
import GroupForm from '~/components/groups/Form.vue';
import Alerters from '~/components/alerters/Alerters.vue';
import AlerterForm from '~/components/alerters/Form.vue';
import Settings from '~/components/login/Settings.vue';

const routes = [
  {
    name: 'home',
    path: '/',
    component: Dashboard,
    meta: { title: 'Dashboard' },
  },
  {
    name: 'outages',
    path: '/outages',
    component: Outages,
    meta: { title: 'Outages' },
  },
  {
    name: 'outages.history',
    path: '/history',
    component: History,
    meta: { title: 'Incident history' },
  },
  {
    name: 'outages.view',
    path: '/outages/:uuid',
    component: Outage,
    meta: { title: 'Outage' },
  },
  {
    name: 'checks',
    path: '/checks',
    component: Checks,
    meta: { title: 'Checks' },
  },
  {
    name: 'checks.new',
    path: '/checks/new',
    component: CheckForm,
    meta: { title: 'New check', action: 'new' },
  },
  {
    name: 'checks.view',
    path: '/checks/:uuid',
    component: Check,
    meta: { title: 'Check' },
  },
  {
    name: 'checks.edit',
    path: '/checks/:uuid/edit',
    component: CheckForm,
    meta: { title: 'Edit check', action: 'edit' },
  },
  {
    name: 'groups',
    path: '/groups',
    component: Groups,
    meta: { title: 'Groups' },
  },
  {
    name: 'groups.new',
    path: '/groups/new',
    component: GroupForm,
    meta: { title: 'New group', action: 'new' },
  },
  {
    name: 'groups.edit',
    path: '/groups/:uuid',
    component: GroupForm,
    meta: { title: 'Edit group', action: 'edit' },
  },
  {
    name: 'alerters',
    path: '/alerters',
    component: Alerters,
    meta: { title: 'Alerters' },
  },
  {
    name: 'alerters.new',
    path: '/alerters/new',
    component: AlerterForm,
    meta: { title: 'New alerter', action: 'new' },
  },
  {
    name: 'alerters.edit',
    path: '/alerters/:uuid',
    component: AlerterForm,
    meta: { title: 'Edit alerter', action: 'edit' },
  },
  {
    name: 'settings',
    path: '/settings',
    component: Settings,
    meta: { title: 'Settings' },
  },
  {
    name: 'logout',
    path: '/-/logout',
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

router.beforeEach((to) => {
  if (to.name === 'logout') {
    store().revokeToken();

    return { name: 'home' };
  }
  return true;
});

export default router;
